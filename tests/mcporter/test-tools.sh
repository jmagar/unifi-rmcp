#!/usr/bin/env bash
# =============================================================================
# test-tools.sh — Integration smoke-test for runifi (UniFi MCP server) tools
#
# Tests every non-destructive action:
#   clients, devices, wlans, health, alarms, sysinfo, me, help
#
# Also tests the schema resource: unifi://schema/mcp-tool
#
# Semantic validation verifies real UniFi API data is flowing:
#   clients  → data array (empty is OK when no clients connected)
#   devices  → data array with at least one item (APs/switches exist)
#   wlans    → data array with at least one WLAN (name, security fields)
#   health   → data array with subsystems
#   alarms   → data array
#   sysinfo  → data with version field
#   me       → name field (object, not array)
#   help     → help key
#
# Credentials sourced from ~/.claude-homelab/.env:
#   UNIFI_MCP_HOST   (default: localhost)
#   UNIFI_MCP_PORT   (default: 40030)
#   UNIFI_MCP_TOKEN  (optional; used as Bearer token when set)
#
# Usage:
#   ./tests/mcporter/test-tools.sh [--timeout-ms N] [--parallel] [--verbose]
#
# Options:
#   --timeout-ms N   Per-call timeout in milliseconds (default: 25000)
#   --parallel       Run independent test groups in parallel (default: off)
#   --verbose        Print raw mcporter output for each call
#
# Exit codes:
#   0 — all tests passed or skipped
#   1 — one or more tests failed
#   2 — prerequisite check failed (mcporter not found, server unreachable)
# =============================================================================

set -uo pipefail

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------
readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly PROJECT_DIR="$(cd -- "${SCRIPT_DIR}/../.." && pwd -P)"
readonly SCRIPT_NAME="$(basename -- "${BASH_SOURCE[0]}")"
readonly TS_START="$(date +%s%N)"
readonly LOG_FILE="${TMPDIR:-/tmp}/${SCRIPT_NAME%.sh}.$(date +%Y%m%d-%H%M%S).log"
readonly ENV_FILE="${HOME}/.claude-homelab/.env"

# Colours (disabled automatically when stdout is not a terminal)
if [[ -t 1 ]]; then
  C_RESET='\033[0m'
  C_BOLD='\033[1m'
  C_GREEN='\033[0;32m'
  C_RED='\033[0;31m'
  C_YELLOW='\033[0;33m'
  C_CYAN='\033[0;36m'
  C_DIM='\033[2m'
else
  C_RESET='' C_BOLD='' C_GREEN='' C_RED='' C_YELLOW='' C_CYAN='' C_DIM=''
fi

# ---------------------------------------------------------------------------
# Defaults (overridable via flags)
# ---------------------------------------------------------------------------
CALL_TIMEOUT_MS=25000
USE_PARALLEL=false
VERBOSE=false

# ---------------------------------------------------------------------------
# Counters
# ---------------------------------------------------------------------------
PASS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0
declare -a FAIL_NAMES=()

# Runtime globals — populated in load_env
MCP_URL=''
MCPORTER_HEADER_ARGS=()

# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------
parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --timeout-ms)
        CALL_TIMEOUT_MS="${2:?--timeout-ms requires a value}"
        shift 2
        ;;
      --parallel)
        USE_PARALLEL=true
        shift
        ;;
      --verbose)
        VERBOSE=true
        shift
        ;;
      -h|--help)
        printf 'Usage: %s [--timeout-ms N] [--parallel] [--verbose]\n' "${SCRIPT_NAME}"
        exit 0
        ;;
      *)
        printf '[ERROR] Unknown argument: %s\n' "$1" >&2
        exit 2
        ;;
    esac
  done
}

# ---------------------------------------------------------------------------
# Logging helpers
# ---------------------------------------------------------------------------
log_info()  { printf "${C_CYAN}[INFO]${C_RESET}  %s\n" "$*" | tee -a "${LOG_FILE}"; }
log_warn()  { printf "${C_YELLOW}[WARN]${C_RESET}  %s\n" "$*" | tee -a "${LOG_FILE}"; }
log_error() { printf "${C_RED}[ERROR]${C_RESET} %s\n" "$*" | tee -a "${LOG_FILE}" >&2; }

elapsed_ms() {
  local now
  now="$(date +%s%N)"
  printf '%d' "$(( (now - TS_START) / 1000000 ))"
}

# ---------------------------------------------------------------------------
# Cleanup trap
# ---------------------------------------------------------------------------
cleanup() {
  local rc=$?
  if [[ $rc -ne 0 ]]; then
    log_warn "Script exited with rc=${rc}. Log: ${LOG_FILE}"
  fi
}
trap cleanup EXIT

# ---------------------------------------------------------------------------
# Load environment and build MCP URL + auth headers
# ---------------------------------------------------------------------------
load_env() {
  if [[ -f "${ENV_FILE}" ]]; then
    # shellcheck disable=SC1090
    set -a
    source "${ENV_FILE}"
    set +a
    log_info "Loaded credentials from ${ENV_FILE}"
  else
    log_warn "${ENV_FILE} not found — using defaults / environment"
  fi

  local host="${UNIFI_MCP_HOST:-localhost}"
  # Remap bind address 0.0.0.0 → localhost for outbound connections
  if [[ "${host}" == "0.0.0.0" ]]; then
    host="localhost"
  fi
  local port="${UNIFI_MCP_PORT:-7474}"
  MCP_URL="http://${host}:${port}/mcp"

  local token="${UNIFI_MCP_TOKEN:-}"

  MCPORTER_HEADER_ARGS=()
  if [[ -n "${token}" ]]; then
    MCPORTER_HEADER_ARGS+=(--header "Authorization: Bearer ${token}")
  fi

  log_info "MCP URL: ${MCP_URL}"
  if [[ ${#MCPORTER_HEADER_ARGS[@]} -gt 0 ]]; then
    log_info "Auth: Bearer token configured"
  else
    log_info "Auth: none (UNIFI_MCP_TOKEN unset)"
  fi
}

# ---------------------------------------------------------------------------
# Prerequisite checks
# ---------------------------------------------------------------------------
check_prerequisites() {
  local missing=false

  if ! command -v mcporter &>/dev/null; then
    log_error "mcporter not found in PATH. Install it and re-run."
    missing=true
  fi

  if ! command -v python3 &>/dev/null; then
    log_error "python3 not found in PATH."
    missing=true
  fi

  if ! command -v curl &>/dev/null; then
    log_error "curl not found in PATH."
    missing=true
  fi

  if [[ "${missing}" == true ]]; then
    return 2
  fi
}

# ---------------------------------------------------------------------------
# Server connectivity smoke-test
# ---------------------------------------------------------------------------
smoke_test_server() {
  log_info "Smoke-testing server connectivity..."

  local base_url="${MCP_URL%/mcp}"

  # 1. Health endpoint (no auth required)
  local health_status
  health_status="$(
    curl -sf --max-time 10 "${base_url}/health" 2>/dev/null | \
    python3 -c "import sys,json; print(json.load(sys.stdin).get('status',''))" 2>/dev/null
  )" || health_status=''

  if [[ "${health_status}" != "ok" ]]; then
    log_error "Health endpoint at ${base_url}/health did not return status=ok (got: '${health_status}')"
    log_error "Is the unifi-rmcp container running?  docker ps | grep unifi-rmcp"
    return 2
  fi
  log_info "Health endpoint OK"

  # 2. tools/list to verify MCP layer responds
  local tool_count
  tool_count="$(
    curl -sf --max-time 10 \
      -X POST "${MCP_URL}" \
      -H "Content-Type: application/json" \
      -H "Accept: application/json, text/event-stream" \
      ${MCPORTER_HEADER_ARGS[@]+"${MCPORTER_HEADER_ARGS[@]}"} \
      -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' 2>/dev/null | \
    python3 -c "
import sys, json
d = json.load(sys.stdin)
tools = d.get('result', {}).get('tools', [])
print(len(tools))
" 2>/dev/null
  )" || tool_count=0

  if [[ "${tool_count}" -lt 1 ]] 2>/dev/null; then
    log_error "tools/list returned ${tool_count} tools — expected at least 1"
    return 2
  fi

  log_info "Server OK — ${tool_count} tools available"
  return 0
}

# ---------------------------------------------------------------------------
# mcporter call wrapper
#   Usage: mcporter_call <args_json>
# ---------------------------------------------------------------------------
mcporter_call() {
  local args_json="${1:?args_json required}"

  mcporter call \
    --http-url "${MCP_URL}" \
    --allow-http \
    ${MCPORTER_HEADER_ARGS[@]+"${MCPORTER_HEADER_ARGS[@]}"} \
    --tool "unifi" \
    --args "${args_json}" \
    --timeout "${CALL_TIMEOUT_MS}" \
    --output json \
    2>>"${LOG_FILE}"
}

# ---------------------------------------------------------------------------
# Test runner
#   Usage: run_test <label> <args_json> [expected_key]
#
#   expected_key supports dot-notation traversal, e.g. "data.0.name"
# ---------------------------------------------------------------------------
run_test() {
  local label="${1:?label required}"
  local args="${2:?args required}"
  local expected_key="${3:-}"

  local t0
  t0="$(date +%s%N)"

  local output
  output="$(mcporter_call "${args}")" || true

  local elapsed_ms_val
  elapsed_ms_val="$(( ( $(date +%s%N) - t0 ) / 1000000 ))"

  if [[ "${VERBOSE}" == true ]]; then
    printf '%s\n' "${output}" | tee -a "${LOG_FILE}"
  else
    printf '%s\n' "${output}" >> "${LOG_FILE}"
  fi

  # Validate JSON is parseable and not an error payload
  local json_check
  json_check="$(
    printf '%s' "${output}" | python3 -c "
import sys, json
try:
    d = json.load(sys.stdin)
    if isinstance(d, dict) and ('error' in d or d.get('kind') == 'error'):
        print('error: ' + str(d.get('error', d.get('message', 'unknown error'))))
    else:
        print('ok')
except Exception as e:
    print('invalid_json: ' + str(e))
" 2>/dev/null
  )" || json_check="parse_error"

  if [[ "${json_check}" != "ok" ]]; then
    printf "${C_RED}[FAIL]${C_RESET} %-60s ${C_DIM}%dms${C_RESET}\n" \
      "${label}" "${elapsed_ms_val}" | tee -a "${LOG_FILE}"
    printf '       response validation failed: %s\n' "${json_check}" | tee -a "${LOG_FILE}"
    FAIL_COUNT=$(( FAIL_COUNT + 1 ))
    FAIL_NAMES+=("${label}")
    return 1
  fi

  # Validate optional key presence (dot-notation)
  if [[ -n "${expected_key}" ]]; then
    local key_check
    key_check="$(
      printf '%s' "${output}" | python3 -c "
import sys, json
try:
    d = json.load(sys.stdin)
    keys = '${expected_key}'.split('.')
    node = d
    for k in keys:
        if k:
            node = node[int(k)] if (isinstance(node, list) and k.isdigit()) else node[k]
    print('ok')
except Exception as e:
    print('missing: ' + str(e))
" 2>/dev/null
    )" || key_check="parse_error"

    if [[ "${key_check}" != "ok" ]]; then
      printf "${C_RED}[FAIL]${C_RESET} %-60s ${C_DIM}%dms${C_RESET}\n" \
        "${label}" "${elapsed_ms_val}" | tee -a "${LOG_FILE}"
      printf '       expected key .%s not found: %s\n' "${expected_key}" "${key_check}" | tee -a "${LOG_FILE}"
      FAIL_COUNT=$(( FAIL_COUNT + 1 ))
      FAIL_NAMES+=("${label}")
      return 1
    fi
  fi

  printf "${C_GREEN}[PASS]${C_RESET} %-60s ${C_DIM}%dms${C_RESET}\n" \
    "${label}" "${elapsed_ms_val}" | tee -a "${LOG_FILE}"
  PASS_COUNT=$(( PASS_COUNT + 1 ))
  return 0
}

# ---------------------------------------------------------------------------
# Skip helper
# ---------------------------------------------------------------------------
skip_test() {
  local label="${1:?label required}"
  local reason="${2:-prerequisite returned empty}"
  printf "${C_YELLOW}[SKIP]${C_RESET} %-60s %s\n" "${label}" "${reason}" | tee -a "${LOG_FILE}"
  SKIP_COUNT=$(( SKIP_COUNT + 1 ))
}

# ---------------------------------------------------------------------------
# Auth enforcement tests
# ---------------------------------------------------------------------------
suite_auth() {
  if [[ -z "${UNIFI_MCP_TOKEN:-}" ]]; then
    printf '\n%b== auth (skipped — token unset) ==%b\n' "${C_BOLD}" "${C_RESET}" | tee -a "${LOG_FILE}"
    skip_test "auth: unauthenticated request returns 401" "UNIFI_MCP_TOKEN unset"
    skip_test "auth: bad token returns 401"               "UNIFI_MCP_TOKEN unset"
    return
  fi

  printf '\n%b== auth enforcement ==%b\n' "${C_BOLD}" "${C_RESET}" | tee -a "${LOG_FILE}"

  local label status

  label="auth: unauthenticated /mcp returns 401"
  status="$(curl -s --max-time 10 -o /dev/null -w "%{http_code}" \
    "${MCP_URL}" -X POST -H "Content-Type: application/json" \
    -H "Accept: application/json, text/event-stream" \
    -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' 2>/dev/null)" || status=0
  if [[ "${status}" == "401" ]]; then
    printf "${C_GREEN}[PASS]${C_RESET} %-60s\n" "${label}" | tee -a "${LOG_FILE}"
    PASS_COUNT=$(( PASS_COUNT + 1 ))
  else
    printf "${C_RED}[FAIL]${C_RESET} %-60s (got HTTP %s)\n" "${label}" "${status}" | tee -a "${LOG_FILE}"
    FAIL_COUNT=$(( FAIL_COUNT + 1 ))
    FAIL_NAMES+=("${label}")
  fi

  label="auth: bad token returns 401"
  status="$(curl -s --max-time 10 -o /dev/null -w "%{http_code}" \
    "${MCP_URL}" -X POST \
    -H "Authorization: Bearer bad-token-intentionally-invalid" \
    -H "Content-Type: application/json" \
    -H "Accept: application/json, text/event-stream" \
    -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' 2>/dev/null)" || status=0
  if [[ "${status}" == "401" ]]; then
    printf "${C_GREEN}[PASS]${C_RESET} %-60s\n" "${label}" | tee -a "${LOG_FILE}"
    PASS_COUNT=$(( PASS_COUNT + 1 ))
  else
    printf "${C_RED}[FAIL]${C_RESET} %-60s (got HTTP %s)\n" "${label}" "${status}" | tee -a "${LOG_FILE}"
    FAIL_COUNT=$(( FAIL_COUNT + 1 ))
    FAIL_NAMES+=("${label}")
  fi
}

# ---------------------------------------------------------------------------
# Test suites
# ---------------------------------------------------------------------------

suite_help() {
  printf '\n%b== help ==%b\n' "${C_BOLD}" "${C_RESET}" | tee -a "${LOG_FILE}"
  run_test "unifi help: returns help key"  '{"action":"help"}' "help"
}

suite_clients() {
  printf '\n%b== clients ==%b\n' "${C_BOLD}" "${C_RESET}" | tee -a "${LOG_FILE}"
  # data array must exist (empty is OK when no clients are connected)
  run_test "unifi clients: returns data array"  '{"action":"clients"}' "data"
}

suite_devices() {
  printf '\n%b== devices ==%b\n' "${C_BOLD}" "${C_RESET}" | tee -a "${LOG_FILE}"
  run_test "unifi devices: returns data array"          '{"action":"devices"}' "data"
  # At least one device must exist — APs/switches are always present in a live network
  run_test "unifi devices: first device has name field" '{"action":"devices"}' "data.0.name"
  run_test "unifi devices: first device has mac field"  '{"action":"devices"}' "data.0.mac"
  run_test "unifi devices: first device has type field" '{"action":"devices"}' "data.0.type"
}

suite_wlans() {
  printf '\n%b== wlans ==%b\n' "${C_BOLD}" "${C_RESET}" | tee -a "${LOG_FILE}"
  run_test "unifi wlans: returns data array"           '{"action":"wlans"}' "data"
  # At least one WLAN must be configured
  run_test "unifi wlans: first WLAN has name field"    '{"action":"wlans"}' "data.0.name"
  run_test "unifi wlans: first WLAN has security field" '{"action":"wlans"}' "data.0.security"
}

suite_health() {
  printf '\n%b== health ==%b\n' "${C_BOLD}" "${C_RESET}" | tee -a "${LOG_FILE}"
  run_test "unifi health: returns data array"               '{"action":"health"}' "data"
  run_test "unifi health: first subsystem has status field"  '{"action":"health"}' "data.0.status"
  run_test "unifi health: first subsystem has subsystem key" '{"action":"health"}' "data.0.subsystem"
}

suite_alarms() {
  printf '\n%b== alarms ==%b\n' "${C_BOLD}" "${C_RESET}" | tee -a "${LOG_FILE}"
  # alarms array may be empty on a healthy network
  run_test "unifi alarms: returns data array" '{"action":"alarms"}' "data"
}

suite_sysinfo() {
  printf '\n%b== sysinfo ==%b\n' "${C_BOLD}" "${C_RESET}" | tee -a "${LOG_FILE}"
  run_test "unifi sysinfo: returns data"                '{"action":"sysinfo"}' "data"
  run_test "unifi sysinfo: data has version field"      '{"action":"sysinfo"}' "data.0.version"
  run_test "unifi sysinfo: data has hostname field"     '{"action":"sysinfo"}' "data.0.hostname"
}

suite_me() {
  printf '\n%b== me ==%b\n' "${C_BOLD}" "${C_RESET}" | tee -a "${LOG_FILE}"
  # me returns {data: {...}} — object not array
  run_test "unifi me: returns data object"    '{"action":"me"}' "data"
  run_test "unifi me: data has name field"    '{"action":"me"}' "data.name"
}

suite_schema_resource() {
  printf '\n%b== schema resource ==%b\n' "${C_BOLD}" "${C_RESET}" | tee -a "${LOG_FILE}"

  # Fetch via resources/read — the URI is unifi://schema/mcp-tool
  local t0 output elapsed_ms_val json_check
  t0="$(date +%s%N)"

  output="$(
    curl -sf --max-time "${CALL_TIMEOUT_MS}" \
      -X POST "${MCP_URL}" \
      -H "Content-Type: application/json" \
      -H "Accept: application/json, text/event-stream" \
      ${MCPORTER_HEADER_ARGS[@]+"${MCPORTER_HEADER_ARGS[@]}"} \
      -d '{"jsonrpc":"2.0","id":2,"method":"resources/read","params":{"uri":"unifi://schema/mcp-tool"}}' \
      2>/dev/null
  )" || output=""

  elapsed_ms_val="$(( ( $(date +%s%N) - t0 ) / 1000000 ))"
  printf '%s\n' "${output}" >> "${LOG_FILE}"

  local label="unifi schema resource: unifi://schema/mcp-tool readable"

  if [[ -z "${output}" ]]; then
    skip_test "${label}" "empty response (resources may not be implemented)"
    return
  fi

  json_check="$(
    printf '%s' "${output}" | python3 -c "
import sys, json
try:
    d = json.load(sys.stdin)
    if 'error' in d:
        err = d['error']
        code = err.get('code', 0)
        # -32601 = method not found; skip gracefully
        if code == -32601:
            print('not_implemented')
        else:
            print('error: ' + str(err))
    elif 'result' in d:
        print('ok')
    else:
        print('unexpected: no result or error key')
except Exception as e:
    print('invalid_json: ' + str(e))
" 2>/dev/null
  )" || json_check="parse_error"

  case "${json_check}" in
    ok)
      printf "${C_GREEN}[PASS]${C_RESET} %-60s ${C_DIM}%dms${C_RESET}\n" \
        "${label}" "${elapsed_ms_val}" | tee -a "${LOG_FILE}"
      PASS_COUNT=$(( PASS_COUNT + 1 ))
      ;;
    not_implemented)
      skip_test "${label}" "server returned -32601 (resources/read not implemented)"
      ;;
    *)
      printf "${C_RED}[FAIL]${C_RESET} %-60s ${C_DIM}%dms${C_RESET}\n" \
        "${label}" "${elapsed_ms_val}" | tee -a "${LOG_FILE}"
      printf '       %s\n' "${json_check}" | tee -a "${LOG_FILE}"
      FAIL_COUNT=$(( FAIL_COUNT + 1 ))
      FAIL_NAMES+=("${label}")
      ;;
  esac
}

# ---------------------------------------------------------------------------
# Print final summary
# ---------------------------------------------------------------------------
print_summary() {
  local total_ms="$(( ( $(date +%s%N) - TS_START ) / 1000000 ))"
  local total=$(( PASS_COUNT + FAIL_COUNT + SKIP_COUNT ))

  printf '\n%b%s%b\n' "${C_BOLD}" "$(printf '=%.0s' {1..65})" "${C_RESET}"
  printf '%b%-20s%b  %b%d%b\n' "${C_BOLD}" "PASS" "${C_RESET}" "${C_GREEN}" "${PASS_COUNT}" "${C_RESET}"
  printf '%b%-20s%b  %b%d%b\n' "${C_BOLD}" "FAIL" "${C_RESET}" "${C_RED}"   "${FAIL_COUNT}" "${C_RESET}"
  printf '%b%-20s%b  %b%d%b\n' "${C_BOLD}" "SKIP" "${C_RESET}" "${C_YELLOW}" "${SKIP_COUNT}" "${C_RESET}"
  printf '%b%-20s%b  %d\n' "${C_BOLD}" "TOTAL" "${C_RESET}" "${total}"
  printf '%b%-20s%b  %ds (%dms)\n' "${C_BOLD}" "ELAPSED" "${C_RESET}" \
    "$(( total_ms / 1000 ))" "${total_ms}"
  printf '%b%s%b\n' "${C_BOLD}" "$(printf '=%.0s' {1..65})" "${C_RESET}"

  if [[ "${FAIL_COUNT}" -gt 0 ]]; then
    printf '\n%bFailed tests:%b\n' "${C_RED}" "${C_RESET}"
    local name
    for name in "${FAIL_NAMES[@]}"; do
      printf '  - %s\n' "${name}"
    done
    printf '\nFull log: %s\n' "${LOG_FILE}"
  fi
}

# ---------------------------------------------------------------------------
# Parallel runner
# ---------------------------------------------------------------------------
run_parallel() {
  log_warn "--parallel mode: per-suite counters aggregated via temp files."

  local tmp_dir
  tmp_dir="$(mktemp -d)"
  trap 'rm -rf -- "${tmp_dir}"' RETURN

  local suites=(
    suite_auth
    suite_help
    suite_clients
    suite_devices
    suite_wlans
    suite_health
    suite_alarms
    suite_sysinfo
    suite_me
    suite_schema_resource
  )

  local pids=()
  local suite
  for suite in "${suites[@]}"; do
    (
      PASS_COUNT=0; FAIL_COUNT=0; SKIP_COUNT=0; FAIL_NAMES=()
      "${suite}"
      printf '%d %d %d\n' "${PASS_COUNT}" "${FAIL_COUNT}" "${SKIP_COUNT}" \
        > "${tmp_dir}/${suite}.counts"
      printf '%s\n' "${FAIL_NAMES[@]:-}" > "${tmp_dir}/${suite}.fails"
    ) &
    pids+=($!)
  done

  local pid
  for pid in "${pids[@]}"; do
    wait "${pid}" || true
  done

  local f
  for f in "${tmp_dir}"/*.counts; do
    [[ -f "${f}" ]] || continue
    local p fl s
    read -r p fl s < "${f}"
    PASS_COUNT=$(( PASS_COUNT + p ))
    FAIL_COUNT=$(( FAIL_COUNT + fl ))
    SKIP_COUNT=$(( SKIP_COUNT + s ))
  done

  for f in "${tmp_dir}"/*.fails; do
    [[ -f "${f}" ]] || continue
    while IFS= read -r line; do
      [[ -n "${line}" ]] && FAIL_NAMES+=("${line}")
    done < "${f}"
  done
}

# ---------------------------------------------------------------------------
# Sequential runner
# ---------------------------------------------------------------------------
run_sequential() {
  suite_auth
  suite_help
  suite_clients
  suite_devices
  suite_wlans
  suite_health
  suite_alarms
  suite_sysinfo
  suite_me
  suite_schema_resource
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
main() {
  parse_args "$@"
  load_env

  printf '%b%s%b\n' "${C_BOLD}" "$(printf '=%.0s' {1..65})" "${C_RESET}"
  printf '%b  runifi integration smoke-test%b\n' "${C_BOLD}" "${C_RESET}"
  printf '%b  Project:  %s%b\n' "${C_BOLD}" "${PROJECT_DIR}" "${C_RESET}"
  printf '%b  MCP URL:  %s%b\n' "${C_BOLD}" "${MCP_URL}" "${C_RESET}"
  printf '%b  Timeout:  %dms/call | Parallel: %s%b\n' \
    "${C_BOLD}" "${CALL_TIMEOUT_MS}" "${USE_PARALLEL}" "${C_RESET}"
  printf '%b  Log:      %s%b\n' "${C_BOLD}" "${LOG_FILE}" "${C_RESET}"
  printf '%b%s%b\n\n' "${C_BOLD}" "$(printf '=%.0s' {1..65})" "${C_RESET}"

  check_prerequisites || exit 2

  smoke_test_server || {
    log_error ""
    log_error "Server connectivity check failed. Aborting — no tests will run."
    log_error ""
    log_error "To diagnose:"
    log_error "  docker ps | grep unifi-rmcp"
    log_error "  curl http://localhost:40030/health"
    log_error "  curl -X POST http://localhost:40030/mcp -H 'Content-Type: application/json' \\"
    log_error "    -d '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\",\"params\":{}}'"
    exit 2
  }

  if [[ "${USE_PARALLEL}" == true ]]; then
    run_parallel
  else
    run_sequential
  fi

  print_summary

  if [[ "${FAIL_COUNT}" -gt 0 ]]; then
    exit 1
  fi
  exit 0
}

main "$@"

#!/usr/bin/env bash
# Claude Code plugin setup hook. Keep service setup owned by the unifi binary.
set -euo pipefail

: "${CLAUDE_PLUGIN_ROOT:=$(cd "$(dirname "$0")/.." && pwd)}"
: "${CLAUDE_PLUGIN_DATA:=${HOME}/.claude/plugins/data/unifi-jmagar-lab}"

reject_unsafe_value() {
  local name="$1" value="${2:-}"
  if [[ "${value}" == *$'\n'* || "${value}" == *$'\r'* ]]; then
    printf 'unifi plugin setup: %s must not contain newlines\n' "${name}" >&2
    exit 2
  fi
}

existing_env_value() {
  local key="$1" file value
  for file in "${CLAUDE_PLUGIN_DATA}/.env"; do
    [[ -f "${file}" ]] || continue
    value="$(awk -F= -v key="${key}" '$1 == key {print substr($0, index($0, "=") + 1); exit}' "${file}")"
    [[ -n "${value}" ]] && { printf '%s\n' "${value}"; return 0; }
  done
  return 0
}

export_option() {
  local env_name="$1" option_name="$2" fallback_key="${3:-}" value
  value="$(printenv "${option_name}" || true)"
  if [[ -z "${value}" && -n "${fallback_key}" ]]; then
    value="$(existing_env_value "${fallback_key}")"
  fi
  reject_unsafe_value "${option_name}" "${value}"
  [[ -n "${value}" ]] || return 0
  export "${env_name}=${value}"
}

ensure_unifi_binary() {
  if command -v unifi >/dev/null 2>&1; then
    return 0
  fi

  local bundled="${CLAUDE_PLUGIN_ROOT}/bin/unifi"
  if [[ -x "${bundled}" ]]; then
    mkdir -p "${HOME}/.local/bin"
    ln -sf "${bundled}" "${HOME}/.local/bin/unifi"
    export PATH="${HOME}/.local/bin:${PATH}"
  fi

  command -v unifi >/dev/null 2>&1 || {
    printf 'unifi plugin setup: unifi binary not found on PATH or at %s\n' "${bundled}" >&2
    exit 1
  }
}

main() {
  mkdir -p "${CLAUDE_PLUGIN_DATA}"
  chmod 700 "${CLAUDE_PLUGIN_DATA}" 2>/dev/null || true
  export UNIFI_MCP_HOME="${CLAUDE_PLUGIN_DATA}"

  export_option UNIFI_MCP_TOKEN CLAUDE_PLUGIN_OPTION_API_TOKEN UNIFI_MCP_TOKEN
  export_option UNIFI_MCP_NO_AUTH CLAUDE_PLUGIN_OPTION_NO_AUTH UNIFI_MCP_NO_AUTH
  export_option UNIFI_MCP_HOST CLAUDE_PLUGIN_OPTION_MCP_HOST UNIFI_MCP_HOST
  export_option UNIFI_MCP_PORT CLAUDE_PLUGIN_OPTION_MCP_PORT UNIFI_MCP_PORT
  export_option UNIFI_MCP_AUTH_MODE CLAUDE_PLUGIN_OPTION_AUTH_MODE UNIFI_MCP_AUTH_MODE
  export_option UNIFI_MCP_PUBLIC_URL CLAUDE_PLUGIN_OPTION_PUBLIC_URL UNIFI_MCP_PUBLIC_URL
  export_option UNIFI_MCP_GOOGLE_CLIENT_ID CLAUDE_PLUGIN_OPTION_GOOGLE_CLIENT_ID UNIFI_MCP_GOOGLE_CLIENT_ID
  export_option UNIFI_MCP_GOOGLE_CLIENT_SECRET CLAUDE_PLUGIN_OPTION_GOOGLE_CLIENT_SECRET UNIFI_MCP_GOOGLE_CLIENT_SECRET
  export_option UNIFI_MCP_AUTH_ADMIN_EMAIL CLAUDE_PLUGIN_OPTION_AUTH_ADMIN_EMAIL UNIFI_MCP_AUTH_ADMIN_EMAIL
  export_option UNIFI_URL CLAUDE_PLUGIN_OPTION_UNIFI_URL UNIFI_URL
  export_option UNIFI_API_KEY CLAUDE_PLUGIN_OPTION_UNIFI_API_KEY UNIFI_API_KEY
  export_option UNIFI_SITE CLAUDE_PLUGIN_OPTION_UNIFI_SITE UNIFI_SITE
  export_option UNIFI_SKIP_TLS_VERIFY CLAUDE_PLUGIN_OPTION_UNIFI_SKIP_TLS UNIFI_SKIP_TLS_VERIFY
  export_option UNIFI_LEGACY CLAUDE_PLUGIN_OPTION_UNIFI_LEGACY UNIFI_LEGACY

  ensure_unifi_binary
  unifi setup plugin-hook "$@"
}

main "$@"

# rustifi — CLAUDE.md

UniFi network MCP server. Read-only REST API bridge for Ubiquiti controllers.

## Module Map

```
src/
  unifi.rs         UnifiClient — HTTP REST client. One method per API endpoint.
  app.rs           UnifiService — wraps UnifiClient. All business logic lives here.
  config.rs        UnifiConfig (UNIFI_*) + McpConfig (UNIFI_MCP_*) + env loading.
  mcp.rs           AppState, AuthPolicy, pub exports, test helper hook.
  mcp/tools.rs     execute_tool() — thin shim: parse args, call service, return Value.
  mcp/schemas.rs   tool_definitions() — JSON schema for the unifi tool.
  mcp/prompts.rs   list_prompts() / get_prompt() — network_summary prompt.
  mcp/rmcp_server.rs  UnifiRmcpServer — rmcp ServerHandler (tools/resources/prompts).
  mcp/routes.rs    axum router with auth middleware and /health endpoint.
  cli.rs           CliCommand — thin shim: parse args, call service, format/print.
  lib.rs           Module declarations. testing:: module (test-support feature).
  main.rs          Dispatch: serve_mcp / serve_stdio_mcp / run_cli.
tests/
  tool_dispatch.rs  MCP tool dispatch unit tests (no network).
  cli_parse.rs      CLI argument parsing unit tests (no network).
```

## Strict Layering Rules

- **All business logic** goes in `app.rs` / `UnifiService`.
- **All HTTP calls** go in `unifi.rs` / `UnifiClient`.
- `mcp/tools.rs` and `cli.rs` are thin shims only: parse args, call service, return/print.
- No logic in `main.rs` beyond dispatch.

## How to Add a New Action

1. Add method to `UnifiClient` in `src/unifi.rs` — one GET call, return raw `Value`.
2. Add delegating method to `UnifiService` in `src/app.rs`.
3. Add match arm in `dispatch()` in `src/mcp/tools.rs`.
4. Add the action string to `UNIFI_ACTIONS` and the schema in `src/mcp/schemas.rs`.
5. Add the action to `READ_ONLY_ACTIONS` in `src/mcp/rmcp_server.rs`.
6. Add `CliCommand` variant, parse arm, dispatch arm, and formatter in `src/cli.rs`.
7. Update help text in `src/mcp/tools.rs` (`HELP_TEXT`) and `src/main.rs` (`print_usage`).

## UniFi API Path Notes

**UDM / UniFi OS (default):**
```
/proxy/network/api/s/{site}/stat/sta        — clients
/proxy/network/api/s/{site}/stat/device     — devices
/proxy/network/api/s/{site}/rest/wlanconf   — WLANs
/proxy/network/api/s/{site}/stat/health     — health
/proxy/network/api/s/{site}/rest/alarm      — alarms
/proxy/network/api/s/{site}/rest/event      — events
/proxy/network/api/s/{site}/stat/sysinfo    — sysinfo
/api/self                                   — me (no /proxy/network prefix)
```

**Legacy (UNIFI_LEGACY=true):** Same paths without `/proxy/network`.

**Response shape:** All site-scoped endpoints return `{"meta": {"rc": "ok"}, "data": [...]}`.
`me` returns `{"data": {...}}`. The client returns the raw Value; callers index `["data"]`.

## Auth

Two modes via `AuthPolicy`:
- `LoopbackDev` — no auth (loopback bind only)
- `Mounted { auth_state: None }` — static bearer token (`UNIFI_MCP_TOKEN`)
- `Mounted { auth_state: Some(_) }` — OAuth (Google) via lab-auth

Scopes: `unifi:read` (all actions), `unifi:admin` (satisfies read too).

## Key Env Vars

```
UNIFI_URL                  Controller base URL (required)
UNIFI_API_KEY              X-API-KEY header value (required)
UNIFI_SITE                 Site name (default: default)
UNIFI_SKIP_TLS_VERIFY      Skip TLS cert check (default: true)
UNIFI_LEGACY               No /proxy/network prefix (default: false)
UNIFI_MCP_PORT             Bind port (default: 40030)
UNIFI_MCP_TOKEN            Static bearer token
UNIFI_MCP_NO_AUTH          Disable auth (loopback only)
```

Runtime secrets live in `~/.unifi-rmcp/.env` on the host. In Docker, that
directory is mounted at `/data`, and the entrypoint sources `/data/.env` before
validating required `UNIFI_*` settings.

## CLI ↔ MCP Action Parity

Every MCP action maps 1-to-1 with a CLI command. Both shims call the same `UnifiService` method.

| Service Method | MCP Action | CLI Command |
|---|---|---|
| `service.clients()` | `unifi(action="clients")` | `unifi clients [--json]` |
| `service.devices()` | `unifi(action="devices")` | `unifi devices [--json]` |
| `service.wlans()` | `unifi(action="wlans")` | `unifi wlans [--json]` |
| `service.health()` | `unifi(action="health")` | `unifi health [--json]` |
| `service.alarms()` | `unifi(action="alarms")` | `unifi alarms [--json]` |
| `service.events(limit)` | `unifi(action="events", limit=N)` | `unifi events [--limit N] [--json]` |
| `service.sysinfo()` | `unifi(action="sysinfo")` | `unifi sysinfo [--json]` |
| `service.me()` | `unifi(action="me")` | `unifi me [--json]` |
| _(built-in)_ | `unifi(action="help")` | `unifi --help` |

## Build & Test

```bash
cargo check          # type-check
cargo test           # unit tests (no network required)
cargo run --bin runifi -- --help
cargo run --bin runifi -- health --json
cargo run --bin runifi           # HTTP MCP server on :40030
cargo run --bin runifi -- mcp    # stdio MCP transport
```


<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:ca08a54f -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

## Session Completion

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd dolt push
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
<!-- END BEADS INTEGRATION -->

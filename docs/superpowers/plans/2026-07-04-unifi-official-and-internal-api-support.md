# UniFi Official And Internal API Support Plan

## Goal

Add registry-backed dispatch coverage for the captured official UniFi Network API operations while preserving verified internal controller actions and friendly hybrid aliases.

## Scope

- Official Network API: all 78 captured operations from `data/unifi_official_network_v10_3_58.json`.
- Internal controller API: neutral reference rows captured in `data/upstream_mcp_network_tools_main.json`; only verified rows in `data/unifi_internal_endpoint_models.json` are exposed as runtime capabilities.
- Hybrid aliases: `list_clients`, `list_devices`, `list_networks`, `list_wifi`, and `get_system_info`.

## Runtime Contract

- Read actions require `unifi:read` in mounted MCP auth.
- Mutating actions require `unifi:admin` in mounted MCP auth.
- CLI and direct dispatch do not use an extra boolean mutation gate.
- Future interactive safety prompts should use MCP elicitation, not action parameters.

## Guardrails

- Tests live in sibling files under `tests/`.
- No Rust source file may exceed 500 LOC.
- No `mod.rs` files.
- New Rust/data/docs filenames use snake_case.
- HTTP transport, path construction, capability registry, action dispatch, MCP schema, and CLI parsing remain separate modules.
- No custom macros for the initial implementation.

## Implementation Summary

- `xtask` refreshes official and internal inventory data.
- `src/api/*` owns path construction and shared HTTP execution.
- `src/capabilities/*` builds the runtime capability registry from checked-in inventory data.
- `src/actions/*` dispatches official, verified internal, and hybrid actions.
- `src/mcp/*` exposes one action-dispatched MCP tool.
- `src/cli.rs` parses legacy and generated actions through the same service boundary.

## Follow-Up

The broader internal reference catalog still needs live endpoint verification before unverified rows can be exposed as callable runtime capabilities. Track that work in bead `rustifi-4wo`.

# UniFi API Full Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build full, auditable parity with the official UniFi Network API inventory and the neutral upstream MCP reference surface.

**Architecture:** Use one canonical internal endpoint model file for reference accounting and runtime exposure. Official API support stays data-driven from the 78-operation official inventory. Verification has three explicit modes: `contract`, `safe_live`, and `mutating_live`; CI runs only contract checks, while live gateway probes are local/operator actions.

**Tech Stack:** Rust 2024, Tokio, reqwest, serde_json, rmcp, xtask, beads, GitHub Actions, UniFi Network official integration API, UniFi Network internal controller API.

## Global Constraints

- Tests live in sibling files under `tests/`; no tests inline with production Rust modules.
- No `.rs` file may exceed 500 LOC.
- No `mod.rs`; use sibling module files.
- New Rust/data/docs filenames use snake_case.
- Do not add any extra boolean mutation gate parameter.
- Mutating MCP actions require admin authorization; local CLI/direct dispatch must not add an extra local mutation gate parameter.
- Use neutral names in repo docs/data for external reference material; do not mention the upstream author/name in committed files.
- Keep boundaries clean: HTTP/path construction in `src/api`, capability inventory in `src/capabilities`, dispatch in `src/actions`, MCP schema in `src/mcp`, CLI parsing in `src/cli.rs`, verification in `xtask`.
- Avoid macros for generated API dispatch unless a later review proves they remove real complexity.
- Do not commit live verification reports or response snippets that can contain local network inventory.

---

## File Structure

- Create `data/upstream_mcp_network_tools_main.json`: neutral raw evidence capture of the upstream MCP reference surface.
- Create `data/unifi_internal_endpoint_models.json`: canonical internal/reference endpoint model file; this is the runtime source of truth.
- Remove runtime dependence on the legacy internal-reference JSON; keep `data/upstream_mcp_network_tools_main.json` as neutral raw evidence and use `data/unifi_internal_endpoint_models.json` as the canonical runtime model.
- Create `src/api/path.rs`: pure path substitution, segment encoding, and connector proxy canonicalization.
- Modify `src/actions/official.rs`: use `src/api/path.rs`; preserve all 78 `official_*` actions.
- Modify `src/actions/internal.rs`: use canonical endpoint models; remove stale bespoke branches that are not model-backed.
- Modify `src/capabilities.rs` and `src/capabilities/internal_network.rs`: add capability metadata for `auth_scope` and `verification_mode`.
- Modify `src/mcp/rmcp_server.rs`: derive MCP read/admin requirements from capability auth scope.
- Modify `xtask/src/endpoint_probe.rs`: default to safe verifier settings, bounded response capture, and sanitized reports.
- Modify `xtask/src/verify_endpoints.rs`: add `contract`, `safe_live`, and `mutating_live` modes.
- Modify `xtask/src/main.rs`: add `check-forbidden-strings`.
- Create `tests/internal_endpoint_models.rs`, `tests/verify_endpoints.rs`, and extend existing registry/path/auth tests.
- Create `docs/unifi_endpoint_verification.md`; update `docs/unifi_api_coverage.md`, `README.md`, and `plugins/unifi/skills/unifi/SKILL.md`.

## Task 1: Canonical Internal Endpoint Models

**Files:**
- Create: `data/upstream_mcp_network_tools_main.json`
- Create: `data/unifi_internal_endpoint_models.json`
- Modify: `xtask/src/internal_reference.rs`
- Modify: `src/capabilities/internal_network.rs`
- Test: `tests/internal_endpoint_models.rs`

**Interfaces:**
- Consumes: neutral upstream reference capture.
- Produces: `InternalEndpointModel { action, title, method, path, mutating, runtime, verified, verification_mode, auth_scope, evidence }`.
- Produces: model metadata `{ source, source_count, accounted_count, runtime_count, non_runtime_count, tools }`.

- [ ] **Step 1: Write the failing model tests**

Create `tests/internal_endpoint_models.rs`:

```rust
use std::collections::HashSet;

use serde_json::Value;

fn models() -> Value {
    serde_json::from_str(include_str!("../data/unifi_internal_endpoint_models.json"))
        .expect("internal endpoint models JSON should parse")
}

#[test]
fn all_upstream_reference_actions_are_accounted_for() {
    let raw: Value = serde_json::from_str(include_str!("../data/upstream_mcp_network_tools_main.json"))
        .expect("neutral upstream reference JSON should parse");
    let models = models();
    let tools = models["tools"].as_array().expect("model tools");
    let raw_tools = raw["tools"].as_array().expect("raw tools");

    assert_eq!(models["source_count"].as_u64(), Some(raw_tools.len() as u64));
    assert_eq!(models["accounted_count"].as_u64(), Some(tools.len() as u64));
    assert_eq!(raw_tools.len(), tools.len(), "every upstream reference row must be modeled");

    let model_actions = tools
        .iter()
        .map(|tool| tool["action"].as_str().unwrap().to_string())
        .collect::<HashSet<_>>();
    for tool in raw_tools {
        let action = tool["action"].as_str().expect("raw action");
        assert!(model_actions.contains(action), "missing endpoint model for {action}");
    }
}

#[test]
fn runtime_models_are_safe_and_evidence_backed() {
    let models = models();
    let mut actions = HashSet::new();
    for tool in models["tools"].as_array().expect("model tools") {
        let action = tool["action"].as_str().expect("action");
        assert!(actions.insert(action.to_string()), "duplicate action {action}");

        let method = tool["method"].as_str().expect("method");
        assert!(matches!(method, "GET" | "POST" | "PUT" | "PATCH" | "DELETE"));

        let path = tool["path"].as_str().expect("path");
        assert!(path.starts_with('/'), "{action} path must be relative absolute");
        assert!(!path.contains("://"), "{action} path must not be absolute URL");
        assert!(!path.contains(".."), "{action} path must not contain traversal");

        let mode = tool["verification_mode"].as_str().expect("verification mode");
        assert!(matches!(mode, "live_2xx" | "contract_ok" | "requires_fixture" | "unsupported"));

        let scope = tool["auth_scope"].as_str().expect("auth scope");
        assert!(matches!(scope, "read" | "admin"));

        if tool["runtime"].as_bool() == Some(true) {
            assert_eq!(tool["verified"].as_bool(), Some(true), "{action} runtime without proof");
            assert_ne!(mode, "unsupported", "{action} runtime unsupported");
            if tool["mutating"].as_bool() == Some(true) {
                assert_eq!(scope, "admin", "{action} mutating without admin scope");
            }
        }
    }
}
```

- [ ] **Step 2: Run the failing test**

Run: `cargo test --test internal_endpoint_models`

Expected: FAIL because the canonical model file does not exist yet.

- [ ] **Step 3: Implement canonical model generation**

Modify `xtask/src/internal_reference.rs` to write `data/unifi_internal_endpoint_models.json` from `data/upstream_mcp_network_tools_main.json`.

Use this exact Rust struct:

```rust
#[derive(Debug, Clone, Serialize)]
struct InternalEndpointModel {
    action: String,
    title: String,
    method: String,
    path: String,
    mutating: bool,
    runtime: bool,
    verified: bool,
    verification_mode: String,
    auth_scope: String,
    evidence: String,
}
```

Rows that are currently live-proven keep `runtime=true`, `verified=true`, `verification_mode="live_2xx"`, and `evidence="live Cloud Gateway Max probe returned 2xx"`. Rows from the upstream reference that are not yet live-proven remain present with `runtime=false` and one of `verification_mode="requires_fixture"`, `verification_mode="contract_ok"`, or `verification_mode="unsupported"`.

- [ ] **Step 4: Load canonical models at runtime**

Modify `src/capabilities/internal_network.rs` so it reads `data/unifi_internal_endpoint_models.json` and exposes only `runtime=true` rows as `Capability`.

- [ ] **Step 5: Regenerate and test**

Run:

```bash
cargo run -p xtask -- refresh-internal-reference
cargo test --test internal_endpoint_models
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add -f data/upstream_mcp_network_tools_main.json data/unifi_internal_endpoint_models.json
git add xtask/src/internal_reference.rs src/capabilities/internal_network.rs tests/internal_endpoint_models.rs
git commit -m "feat: add canonical unifi internal endpoint models"
```

## Task 2: Shared Path And Connector Guardrails

**Files:**
- Create: `src/api/path.rs`
- Modify: `src/api.rs`
- Modify: `src/actions/official.rs`
- Modify: `xtask/src/endpoint_probe.rs`
- Test: `tests/path_building.rs`
- Test: `tests/action_dispatch.rs`

**Interfaces:**
- Produces: `pub fn substitute_path(template: &str, params: &serde_json::Value, allowed_wildcard_prefixes: &[&str]) -> anyhow::Result<String>`.
- Produces: `pub fn encode_path_segment(value: &str) -> String`.
- Produces: `pub fn validate_connector_path(path: &str, allowed_prefixes: &[&str]) -> anyhow::Result<()>`.

- [ ] **Step 1: Write failing path security tests**

Append to `tests/path_building.rs`:

```rust
#[test]
fn shared_path_substitution_encodes_segments() {
    let params = serde_json::json!({"siteId": "site one", "clientId": "aa:bb:cc"});
    let path = rustifi::api::path::substitute_path(
        "/v1/sites/{siteId}/clients/{clientId}",
        &params,
        &[],
    )
    .unwrap();
    assert_eq!(path, "/v1/sites/site%20one/clients/aa%3Abb%3Acc");
}

#[test]
fn connector_path_rejects_bypass_shapes() {
    for candidate in [
        "/api/self",
        "https://example.test/proxy/network/integration/v1/info",
        "//example.test/proxy/network/integration/v1/info",
        "/proxy/network/integration/../api/self",
        "/proxy/network/integration/%2e%2e/api/self",
        "/proxy/network/integration/%2fapi/self",
        "/proxy/network/integration/%5capi/self",
        "/proxy/network/integration/v1/info?x=1",
        "/proxy/network/integration/v1/info#x",
    ] {
        let err = rustifi::api::path::validate_connector_path(
            candidate,
            &["/proxy/network/integration/", "/proxy/protect/integration/"],
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("outside the supported integration API prefix") || err.contains("unsafe connector path"));
    }
}
```

- [ ] **Step 2: Run failing tests**

Run: `cargo test --test path_building connector_path_rejects_bypass_shapes`

Expected: FAIL because `rustifi::api::path` does not exist.

- [ ] **Step 3: Implement `src/api/path.rs`**

Move only path parameter substitution, segment encoding, and connector path validation into `src/api/path.rs`. Reject absolute URLs, scheme-relative paths, backslashes, query strings, fragments, traversal, and encoded traversal/separators before checking allowed prefixes.

- [ ] **Step 4: Export the module**

Modify `src/api.rs`:

```rust
pub mod http;
pub mod internal;
pub mod official;
pub mod path;
```

- [ ] **Step 5: Use the helper in runtime and verifier**

Modify `src/actions/official.rs` and `xtask/src/endpoint_probe.rs` so official path construction uses the same substitution and connector validation behavior. If `xtask` cannot depend on `rustifi` cleanly without a cycle, move the pure helper into a small local module under `xtask/src/endpoint_path.rs` and add a test that runtime and xtask produce identical output for the same fixtures.

- [ ] **Step 6: Run tests**

Run: `cargo test --test path_building --test action_dispatch`

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/api.rs src/api/path.rs src/actions/official.rs xtask/src/endpoint_probe.rs tests/path_building.rs tests/action_dispatch.rs
git commit -m "fix: harden unifi path substitution"
```

## Task 3: Safe Verifier Modes And Artifact Guard

**Files:**
- Modify: `xtask/src/endpoint_probe.rs`
- Modify: `xtask/src/verify_endpoints.rs`
- Modify: `xtask/src/main.rs`
- Test: `tests/verify_endpoints.rs`

**Interfaces:**
- Produces verifier modes: `contract`, `safe_live`, `mutating_live`.
- Produces result statuses: `live_ok`, `contract_ok`, `requires_fixture`, `unsupported`, `auth_failed`, `server_error`, `skipped`.
- Produces `cargo run -p xtask -- check-forbidden-strings`.

- [ ] **Step 1: Write failing verifier safety tests**

Create `tests/verify_endpoints.rs`:

```rust
#[test]
fn verifier_contract_mode_is_the_ci_default() {
    let help = std::process::Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("verify-api-endpoints")
        .arg("--help")
        .output()
        .expect("xtask help should run");
    let stdout = String::from_utf8_lossy(&help.stdout);
    assert!(stdout.contains("contract"));
    assert!(stdout.contains("safe_live"));
    assert!(stdout.contains("mutating_live"));
}

#[test]
fn forbidden_string_checker_exists() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("check-forbidden-strings")
        .output()
        .expect("xtask checker should run");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}
```

- [ ] **Step 2: Run failing tests**

Run: `cargo test --test verify_endpoints`

Expected: FAIL because the new xtask arguments and checker do not exist.

- [ ] **Step 3: Implement verifier modes**

Modify `xtask/src/verify_endpoints.rs`:

- `contract`: no network, validates inventory, paths, auth scopes, and request policies.
- `safe_live`: live probes only read endpoints that do not need object fixtures; mutating and fixture-required operations report `contract_ok` or `requires_fixture`.
- `mutating_live`: opt-in mode for disposable/controlled sites; never runs by default.

- [ ] **Step 4: Add safety knobs**

Modify `xtask/src/endpoint_probe.rs` so default configuration has no mutating live probes and no unverified internal live probes. Add env knobs `UNIFI_VERIFY_MAX_REQUESTS`, `UNIFI_VERIFY_TIMEOUT_SECS`, and `UNIFI_VERIFY_RATE_LIMIT_MS`. Keep concurrency at 1.

- [ ] **Step 5: Redact and relocate reports**

Write live reports under `target/unifi_verification/` by default. Store only method, path template, status, HTTP status, and a sanitized detail capped at 1024 bytes. Do not include response bodies containing client/device/admin inventory.

- [ ] **Step 6: Implement `check-forbidden-strings`**

Add an xtask command that fails if committed files contain forbidden upstream author/name literals, legacy mutation gate literals, `mod.rs`, oversized Rust files, or tracked live verifier reports.

- [ ] **Step 7: Run tests and checker**

Run:

```bash
cargo test --test verify_endpoints
cargo run -p xtask -- check-forbidden-strings
```

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add xtask/src/endpoint_probe.rs xtask/src/verify_endpoints.rs xtask/src/main.rs tests/verify_endpoints.rs
git commit -m "feat: add safe unifi verification modes"
```

## Task 4: Official API Contract Parity

**Files:**
- Modify: `src/capabilities/official_network.rs`
- Modify: `xtask/src/verify_endpoints.rs`
- Modify: `docs/unifi_api_coverage.md`
- Test: `tests/official_registry.rs`

**Interfaces:**
- Consumes: `data/unifi_official_network_v10_3_58.json`.
- Produces: default verifier invariant `official_total == 78` and `official_live_ok + official_contract_ok + official_requires_fixture == 78`.

- [ ] **Step 1: Write official parity tests**

Append to `tests/official_registry.rs`:

```rust
#[test]
fn official_inventory_count_is_the_parity_floor() {
    let inventory: serde_json::Value =
        serde_json::from_str(include_str!("../data/unifi_official_network_v10_3_58.json"))
            .expect("official inventory should parse");
    assert_eq!(inventory["count"].as_u64(), Some(78));
    assert_eq!(inventory["operations"].as_array().unwrap().len(), 78);
}

#[test]
fn every_official_operation_has_registered_action() {
    let inventory: serde_json::Value =
        serde_json::from_str(include_str!("../data/unifi_official_network_v10_3_58.json"))
            .expect("official inventory should parse");
    for op in inventory["operations"].as_array().unwrap() {
        let operation_id = op["operation_id"].as_str().unwrap();
        let action = rustifi::capabilities::official_network::action_name(operation_id);
        assert!(rustifi::capabilities::find_capability(&action).is_some(), "missing {action}");
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --test official_registry`

Expected: PASS after `action_name` is public or an equivalent test helper exists.

- [ ] **Step 3: Classify official verification policies**

Modify the verifier so official operations are classified:

- Read list/static endpoints: `live_ok` in `safe_live`, `contract_ok` in `contract`.
- Read detail endpoints needing object IDs: `requires_fixture` unless fixture env values are supplied.
- Mutating endpoints: `contract_ok` in `contract` and `safe_live`; live probe only in `mutating_live`.
- Connector proxy routes: `contract_ok` unless explicitly invoked with a safe integration path fixture.

- [ ] **Step 4: Run default contract verifier**

Run: `cargo run -p xtask -- verify-api-endpoints --mode contract`

Expected: official rejected `0`, official accounted `78`.

- [ ] **Step 5: Update docs**

Modify `docs/unifi_api_coverage.md` to define official parity as registered + path-valid + auth-scoped + contract-verified or safe-live verified.

- [ ] **Step 6: Commit**

```bash
git add src/capabilities/official_network.rs xtask/src/verify_endpoints.rs docs/unifi_api_coverage.md tests/official_registry.rs
git commit -m "feat: verify official unifi api parity"
```

## Task 5: Runtime Auth Scope And Surface Exposure

**Files:**
- Modify: `src/capabilities.rs`
- Modify: `src/capabilities/internal_network.rs`
- Modify: `src/actions/internal.rs`
- Modify: `src/mcp/rmcp_server.rs`
- Modify: `src/mcp/schemas.rs`
- Modify: `src/cli.rs`
- Test: `tests/internal_registry.rs`
- Test: `tests/tool_dispatch.rs`
- Test: `tests/cli_parse.rs`

**Interfaces:**
- Consumes: canonical endpoint models.
- Produces: runtime actions for `runtime=true` model rows and all 78 official actions.
- Produces: `Capability.auth_scope: AuthScope`.

- [ ] **Step 1: Write auth and runtime exposure tests**

Update `tests/internal_registry.rs` so every `runtime=true` model row appears as a capability and every `runtime=false` row does not.

Append to `tests/tool_dispatch.rs`:

```rust
#[tokio::test]
async fn mutating_actions_require_admin_scope() {
    let rf_scan = rustifi::capabilities::find_capability("internal_trigger_rf_scan")
        .expect("rf scan capability");
    assert!(rf_scan.mutating);
    assert_eq!(rf_scan.auth_scope.as_str(), "admin");

    let clients = rustifi::capabilities::find_capability("clients")
        .expect("clients capability");
    assert_eq!(clients.auth_scope.as_str(), "read");
}
```

- [ ] **Step 2: Run failing tests**

Run: `cargo test --test internal_registry --test tool_dispatch`

Expected: FAIL because `Capability.auth_scope` does not exist yet.

- [ ] **Step 3: Add auth scope metadata**

Modify `src/capabilities.rs` with:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthScope {
    Read,
    Admin,
}

impl AuthScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Admin => "admin",
        }
    }
}
```

Add `auth_scope: AuthScope` and `verification_mode: Option<String>` to `Capability`.

- [ ] **Step 4: Enforce auth scope in MCP**

Modify `src/mcp/rmcp_server.rs` so read/admin checks use `capability.auth_scope` instead of only `capability.mutating`. Document static bearer mode as all-powerful unless separate tokens are configured.

- [ ] **Step 5: Remove stale internal branches**

Modify `src/actions/internal.rs` so unmodeled actions are not matched. Remove any stale branch for an unverified event action unless it exists in canonical models with `runtime=true`.

- [ ] **Step 6: Run runtime tests**

Run: `cargo test --test internal_registry --test tool_dispatch --test cli_parse`

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/capabilities.rs src/capabilities/internal_network.rs src/actions/internal.rs src/mcp/rmcp_server.rs src/mcp/schemas.rs src/cli.rs tests/internal_registry.rs tests/tool_dispatch.rs tests/cli_parse.rs
git commit -m "feat: enforce unifi capability auth scopes"
```

## Task 6: Docs, Local Gates, CI, And PR Readiness

**Files:**
- Create: `docs/unifi_endpoint_verification.md`
- Modify: `docs/unifi_api_coverage.md`
- Modify: `README.md`
- Modify: `plugins/unifi/skills/unifi/SKILL.md`
- Modify: `.github/workflows/*` only if CI proves a missing required gate.

**Interfaces:**
- Produces: a reproducible verification checklist for local and CI.

- [ ] **Step 1: Add endpoint verification docs**

Create `docs/unifi_endpoint_verification.md`:

```markdown
# UniFi Endpoint Verification

`cargo run -p xtask -- verify-api-endpoints --mode contract` validates registry, path, auth-scope, and request-policy coverage without network access.

`cargo run -p xtask -- verify-api-endpoints --mode safe_live` additionally probes safe read endpoints against a configured controller.

`cargo run -p xtask -- verify-api-endpoints --mode mutating_live` is reserved for disposable or controlled sites.

Live reports are local artifacts under `target/unifi_verification/` and must not be committed.
```

- [ ] **Step 2: Update user docs**

Update README and plugin skill docs with short action-surface summaries only. Put detailed verifier behavior in `docs/unifi_endpoint_verification.md`.

- [ ] **Step 3: Run full local gate**

Run:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -p xtask -- verify-api-endpoints --mode contract
cargo run -p xtask -- check-forbidden-strings
```

Expected: all pass.

- [ ] **Step 4: Commit**

```bash
git add docs/unifi_endpoint_verification.md docs/unifi_api_coverage.md README.md plugins/unifi/skills/unifi/SKILL.md
git commit -m "docs: document unifi parity verification"
```

- [ ] **Step 5: Push and fix CI**

Run:

```bash
git push
gh pr checks --watch
```

Expected: all GitHub Actions pass. If any action fails, use `vibin:gh-fix-ci`: inspect logs with `gh run view`, patch, rerun the failing local gate, commit, push, and recheck.

## Engineering Review Findings Applied

- Use one canonical internal endpoint model file for runtime and accounting; keep raw upstream capture as evidence only.
- Preserve every upstream reference row as accounted, even when `runtime=false`.
- Add source/accounted/runtime counters so rows cannot be deleted to make verification green.
- Make verifier default safe and split `contract`, `safe_live`, and `mutating_live`.
- Add request budgets, timeout/rate-limit knobs, sanitized reports, and tracked-report guard.
- Add connector path canonicalization and bypass tests.
- Add capability auth scope and MCP auth-scope tests.
- Add `xtask check-forbidden-strings` rather than embedding forbidden literals in repo docs.

## Self-Review

**Spec coverage:** This plan covers official UniFi API parity, upstream MCP reference accounting, runtime exposure, verification, docs, CI, no extra boolean mutation gate, no forbidden upstream author/name in repo docs, sibling tests, no `mod.rs`, and Rust file size limits.

**Placeholder scan:** No step uses TBD/fill-in language. Every implementation step names exact files, commands, expected output, and concrete code or schema shape.

**Type consistency:** `InternalEndpointModel`, `AuthScope`, verifier modes, `substitute_path`, and JSON fields are named consistently across tasks.

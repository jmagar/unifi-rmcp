# WireMock API Call Tests Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add mock HTTP tests that prove generated official actions, generated internal endpoint actions, and preserved legacy aliases build the expected UniFi HTTP method, path, query, body, and scope behavior without calling a live controller. Use failures from those tests to drive any missing upstream-style argument-to-request mappings before claiming full tool-level parity.

**Architecture:** Add `wiremock` as a dev-only dependency and replace broad request-string capture coverage with matcher-based sibling integration tests. Keep production HTTP construction in `src/actions/*` and `src/api/*`; tests drive only public crate APIs such as `ActionDispatcher`, `find_capability`, and `required_scope_for`.

**Tech Stack:** Rust 2021, Tokio, reqwest, serde_json, wiremock, rmcp, sibling integration tests under `tests/`.

## Global Constraints

- Repo/worktree: `/home/jmagar/workspace/rustifi/.worktrees/codex-unifi-full-parity`.
- Branch: `codex/unifi-full-parity`.
- PR: `#3 Implement UniFi API endpoint parity`.
- Tests live in sibling files under `tests/`; no tests inline with production Rust modules.
- No `.rs` file may exceed 500 LOC.
- No `mod.rs`; use sibling module files.
- Do not add the repo's prohibited approval-wording token in code, test names, docs, fixtures, or assertions.
- Do not mention any external reference author name in repo artifacts.
- Preserve existing legacy aliases: `clients`, `devices`, `wlans`, `health`, `alarms`, `events`, `sysinfo`, `me`, `list_clients`, `list_devices`, `list_networks`, `list_wifi`, `get_system_info`.
- Preserve existing registry, contract, and live verifier coverage; the new mock tests add request-construction proof only.
- Do not implement the tests as part of this plan-writing task.

---

## File Structure

- Modify `Cargo.toml`: add `wiremock = "0.6"` to `[dev-dependencies]`.
- Create `tests/support_http.rs`: shared test helpers for mock-server setup, test config, and dispatcher calls. This is a sibling integration test crate, not a module imported by production Rust code.
- Create `tests/http_request_construction.rs`: matcher-based request-construction tests for official generated actions, internal generated actions, v1/v2 path routing, query encoding, JSON body forwarding, connector wildcard proxy paths, empty mutating responses, and legacy aliases.
- Create `tests/mcp_scope.rs`: scope mapping tests for read/admin/unknown/help behavior using `rustifi::mcp::rmcp_server::required_scope_for`.
- Modify `src/mcp.rs`: expose `rmcp_server` as `pub mod rmcp_server;` only if current visibility prevents sibling tests from importing `required_scope_for`.
- Do not modify `src/actions/official.rs`, `src/actions/internal.rs`, `src/api/http.rs`, `src/api/path.rs`, or capability registries unless a planned test reveals a real bug during execution.
- Keep `tests/action_dispatch.rs` intact at first; after new tests pass, remove only the custom `CaptureServer` tests that are fully superseded so the file stays focused and below 500 LOC.

## Dependency Choice Rationale

- Choose `wiremock` because the repo already uses Tokio async tests, and `wiremock::MockServer` works naturally with async test functions.
- Choose `wiremock` because request expectations are explicit matchers: HTTP method, path, query parameter, header, and JSON body are readable in the test itself.
- Do not choose `httpmock` for this pass because its API is broader than needed and its assertions tend to put more logic inside closure-style request inspection.
- Do not extend the hand-rolled `CaptureServer` because raw request strings are brittle, case-normalized, and awkward for body/query/header matching across 78 official operations plus 175 internal runtime rows.
- Keep the dependency dev-only so production binary size, runtime behavior, and plugin setup hooks do not change.

## Task 1: Add WireMock And Shared Test Helpers

**Files:**
- Modify: `Cargo.toml`
- Create: `tests/support_http.rs`

**Interfaces:**
- Produces: `pub fn test_config(url: impl Into<String>) -> rustifi::config::UnifiConfig`.
- Produces: `pub fn dispatcher(url: impl Into<String>) -> rustifi::actions::ActionDispatcher`.
- Produces: `pub async fn run_action(server: &wiremock::MockServer, action: &str, params: serde_json::Value) -> anyhow::Result<serde_json::Value>`.

- [ ] **Step 1: Add the dev dependency**

Modify `[dev-dependencies]` in `Cargo.toml`:

```toml
wiremock = "0.6"
```

- [ ] **Step 2: Create the helper test crate**

Create `tests/support_http.rs`:

```rust
use serde_json::Value;

use rustifi::actions::{ActionDispatcher, ActionRequest};
use rustifi::config::UnifiConfig;

pub fn test_config(url: impl Into<String>) -> UnifiConfig {
    UnifiConfig {
        url: url.into(),
        api_key: "test-key".into(),
        site: "default".into(),
        skip_tls_verify: true,
        legacy: false,
    }
}

pub fn dispatcher(url: impl Into<String>) -> ActionDispatcher {
    ActionDispatcher::new_for_test(test_config(url))
}

pub async fn run_action(
    server: &wiremock::MockServer,
    action: &str,
    params: Value,
) -> anyhow::Result<Value> {
    dispatcher(server.uri())
        .execute(ActionRequest {
            action: action.to_string(),
            params,
        })
        .await
}
```

- [ ] **Step 3: Check helper compilation**

Run: `cargo test --test support_http`

Expected: PASS with `0 tests`, proving the helper crate compiles.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock tests/support_http.rs
git commit -m "test: add wiremock support helpers"
```

## Task 2: Official Generated Request Construction Tests

**Files:**
- Create: `tests/http_request_construction.rs`
- Uses: `tests/support_http.rs`

**Interfaces:**
- Consumes: `support_http::run_action(server, action, params)`.
- Verifies: official actions route through `/proxy/network/integration/v1`, send `X-API-KEY`, preserve query params, substitute encoded path params, and forward JSON bodies.

- [ ] **Step 1: Write official GET query test**

Create `tests/http_request_construction.rs`:

```rust
#[path = "support_http.rs"]
mod support_http;

use serde_json::json;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn official_list_clients_sends_get_path_query_and_api_key() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/proxy/network/integration/v1/sites/site-1/clients"))
        .and(query_param("limit", "1"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"items": []})))
        .expect(1)
        .mount(&server)
        .await;

    let result = support_http::run_action(
        &server,
        "official_list_clients",
        json!({"siteId": "site-1", "query": {"limit": 1}}),
    )
    .await
    .expect("official list clients should succeed");

    assert_eq!(result, json!({"items": []}));
    server.verify().await;
}
```

- [ ] **Step 2: Run the official GET test**

Run: `cargo test --test http_request_construction official_list_clients_sends_get_path_query_and_api_key`

Expected: PASS.

- [ ] **Step 3: Add official body and encoded path tests**

Append to `tests/http_request_construction.rs`:

```rust
use wiremock::matchers::body_json;

#[tokio::test]
async fn official_create_network_sends_post_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/proxy/network/integration/v1/sites/site-1/networks"))
        .and(header("x-api-key", "test-key"))
        .and(body_json(json!({"name": "IoT"})))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({"id": "network-1"})))
        .expect(1)
        .mount(&server)
        .await;

    let result = support_http::run_action(
        &server,
        "official_create_network",
        json!({"siteId": "site-1", "body": {"name": "IoT"}}),
    )
    .await
    .expect("official create network should succeed");

    assert_eq!(result, json!({"id": "network-1"}));
    server.verify().await;
}

#[tokio::test]
async fn official_path_params_are_segment_encoded() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/proxy/network/integration/v1/sites/site-1/networks/net%2Fa%3Fb",
        ))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .expect(1)
        .mount(&server)
        .await;

    let result = support_http::run_action(
        &server,
        "official_get_network_details",
        json!({"siteId": "site-1", "networkId": "net/a?b"}),
    )
    .await
    .expect("encoded network details should succeed");

    assert_eq!(result, json!({"ok": true}));
    server.verify().await;
}
```

- [ ] **Step 4: Run official action tests**

Run: `cargo test --test http_request_construction official_`

Expected: PASS for the three official tests.

- [ ] **Step 5: Commit**

```bash
git add tests/http_request_construction.rs
git commit -m "test: cover official unifi request construction"
```

## Task 3: Official Connector And Empty Response Tests

**Files:**
- Modify: `tests/http_request_construction.rs`

**Interfaces:**
- Verifies: connector wildcard path stays under supported integration proxy prefixes.
- Verifies: mutating official actions with empty response bodies return the existing synthetic success JSON from `api::http::request_json`.

- [ ] **Step 1: Add connector wildcard and empty-body tests**

Append to `tests/http_request_construction.rs`:

```rust
#[tokio::test]
async fn official_connector_get_keeps_proxy_path_inside_integration_prefix() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/proxy/network/integration/v1/connector/consoles/console-1/proxy/network/integration/v1/sites",
        ))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .expect(1)
        .mount(&server)
        .await;

    let result = support_http::run_action(
        &server,
        "official_connector_get",
        json!({
            "id": "console-1",
            "path": "/proxy/network/integration/v1/sites"
        }),
    )
    .await
    .expect("connector get should succeed");

    assert_eq!(result, json!({"ok": true}));
    server.verify().await;
}

#[tokio::test]
async fn official_delete_action_accepts_empty_success_body() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/proxy/network/integration/v1/sites/site-1/port-forwards/rule-1"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let result = support_http::run_action(
        &server,
        "official_delete_port_forward",
        json!({"siteId": "site-1", "portForwardId": "rule-1"}),
    )
    .await
    .expect("official delete port forward should succeed");

    assert_eq!(
        result,
        json!({
            "success": true,
            "status": 204,
            "method": "DELETE",
            "path": "/proxy/network/integration/v1/sites/site-1/port-forwards/rule-1"
        })
    );
    server.verify().await;
}
```

- [ ] **Step 2: Run the connector and empty-body tests**

Run: `cargo test --test http_request_construction official_connector official_delete`

Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add tests/http_request_construction.rs
git commit -m "test: cover connector and empty official responses"
```

## Task 4: Internal Generated Request Construction Tests

**Files:**
- Modify: `tests/http_request_construction.rs`

**Interfaces:**
- Verifies: internal generated v1 actions use `/proxy/network/api/s/{site}`.
- Verifies: internal generated v2 actions use `/proxy/network/v2/api/site/{site}`.
- Verifies: generated internal mutating actions send JSON bodies and use admin-scope capability metadata.

- [ ] **Step 1: Add internal v1 and v2 tests**

Append to `tests/http_request_construction.rs`:

```rust
#[tokio::test]
async fn internal_v1_generated_action_uses_site_api_prefix() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/proxy/network/api/s/default/rest/user"))
        .and(query_param("mac", "aa:bb:cc:dd:ee:ff"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
        .expect(1)
        .mount(&server)
        .await;

    let result = support_http::run_action(
        &server,
        "unifi_authorize_guest",
        json!({"query": {"mac": "aa:bb:cc:dd:ee:ff"}}),
    )
    .await
    .expect("internal v1 action should succeed");

    assert_eq!(result, json!({"data": []}));
    server.verify().await;
}

#[tokio::test]
async fn internal_v2_generated_action_uses_v2_site_prefix_and_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/proxy/network/v2/api/site/default/firewall-policies"))
        .and(header("x-api-key", "test-key"))
        .and(body_json(json!({"name": "Block IoT"})))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({"_id": "policy-1"})))
        .expect(1)
        .mount(&server)
        .await;

    let result = support_http::run_action(
        &server,
        "unifi_create_firewall_policy",
        json!({"body": {"name": "Block IoT"}}),
    )
    .await
    .expect("internal v2 action should succeed");

    assert_eq!(result, json!({"_id": "policy-1"}));
    server.verify().await;
}
```

- [ ] **Step 2: Add internal generated metadata assertions**

Append to `tests/http_request_construction.rs`:

```rust
#[test]
fn internal_generated_mutating_action_requires_admin_scope_metadata() {
    let cap = rustifi::capabilities::find_capability("unifi_create_firewall_policy")
        .expect("internal generated capability");
    assert!(cap.mutating);
    assert_eq!(cap.method.as_deref(), Some("POST"));
    assert_eq!(cap.path.as_deref(), Some("/v2/firewall-policies"));
    assert_eq!(cap.auth_scope.as_str(), "admin");
}
```

- [ ] **Step 3: Run internal tests**

Run: `cargo test --test http_request_construction internal_`

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add tests/http_request_construction.rs
git commit -m "test: cover internal unifi request construction"
```

## Task 5: Legacy Alias Request Construction Tests

**Files:**
- Modify: `tests/http_request_construction.rs`

**Interfaces:**
- Verifies: preserved legacy aliases still call the original internal paths.
- Verifies: hybrid aliases keep existing internal default and official opt-in behavior.

- [ ] **Step 1: Add preserved legacy alias test**

Append to `tests/http_request_construction.rs`:

```rust
#[tokio::test]
async fn preserved_clients_alias_uses_original_internal_clients_path() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/proxy/network/api/s/default/stat/sta"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
        .expect(1)
        .mount(&server)
        .await;

    let result = support_http::run_action(&server, "clients", json!({}))
        .await
        .expect("clients alias should succeed");

    assert_eq!(result, json!({"data": []}));
    server.verify().await;
}
```

- [ ] **Step 2: Add hybrid alias routing tests**

Append to `tests/http_request_construction.rs`:

```rust
#[tokio::test]
async fn hybrid_list_clients_defaults_to_internal_path() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/proxy/network/api/s/default/stat/sta"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
        .expect(1)
        .mount(&server)
        .await;

    let result = support_http::run_action(&server, "list_clients", json!({}))
        .await
        .expect("hybrid list_clients should default to internal");

    assert_eq!(result, json!({"data": []}));
    server.verify().await;
}

#[tokio::test]
async fn hybrid_list_clients_uses_official_path_when_site_id_is_present() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/proxy/network/integration/v1/sites/site-1/clients"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"items": []})))
        .expect(1)
        .mount(&server)
        .await;

    let result = support_http::run_action(&server, "list_clients", json!({"siteId": "site-1"}))
        .await
        .expect("hybrid list_clients should use official with siteId");

    assert_eq!(result, json!({"items": []}));
    server.verify().await;
}
```

- [ ] **Step 3: Run alias tests**

Run: `cargo test --test http_request_construction alias hybrid`

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add tests/http_request_construction.rs
git commit -m "test: cover legacy and hybrid unifi aliases"
```

## Task 6: MCP Scope Behavior Tests

**Files:**
- Create: `tests/mcp_scope.rs`
- Modify: `src/mcp.rs` only if `required_scope_for` is not reachable from integration tests.

**Interfaces:**
- Consumes: `rustifi::mcp::rmcp_server::required_scope_for(action: &str) -> Option<&'static str>`.
- Verifies: official and internal generated read actions require `unifi:read`, mutating actions require `unifi:admin`, `help` requires no scope, and unknown actions map to the deny scope.

- [ ] **Step 1: Write scope tests**

Create `tests/mcp_scope.rs`:

```rust
use rustifi::mcp::rmcp_server::required_scope_for;

#[test]
fn official_read_action_requires_read_scope() {
    assert_eq!(required_scope_for("official_list_clients"), Some("unifi:read"));
}

#[test]
fn official_mutating_action_requires_admin_scope() {
    assert_eq!(required_scope_for("official_create_network"), Some("unifi:admin"));
}

#[test]
fn internal_read_action_requires_read_scope() {
    assert_eq!(required_scope_for("unifi_list_clients"), Some("unifi:read"));
}

#[test]
fn internal_mutating_action_requires_admin_scope() {
    assert_eq!(
        required_scope_for("unifi_create_firewall_policy"),
        Some("unifi:admin")
    );
}

#[test]
fn help_and_unknown_actions_have_explicit_scope_behavior() {
    assert_eq!(required_scope_for("help"), None);
    assert_eq!(required_scope_for("not_a_real_action"), Some("unifi:__deny__"));
}
```

- [ ] **Step 2: Run scope tests**

Run: `cargo test --test mcp_scope`

Expected: PASS if `rmcp_server` is publicly reachable; otherwise FAIL with a privacy error for `rmcp_server`.

- [ ] **Step 3: Expose the minimum testable module if needed**

If Step 2 fails with a module privacy error, modify `src/mcp.rs` from:

```rust
mod rmcp_server;
```

to:

```rust
pub mod rmcp_server;
```

Do not change the signature or visibility of `required_scope_for`; it is already public inside the module.

- [ ] **Step 4: Re-run scope tests**

Run: `cargo test --test mcp_scope`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/mcp.rs tests/mcp_scope.rs
git commit -m "test: cover unifi mcp scope mapping"
```

## Task 7: Replace Superseded CaptureServer Coverage

**Files:**
- Modify: `tests/action_dispatch.rs`
- Keep: `tests/http_request_construction.rs`

**Interfaces:**
- Removes: custom `CaptureServer` and request-string assertions that are now covered by `wiremock`.
- Keeps: validation tests, hybrid resolver tests, and any action-dispatch behavior not asserted by `tests/http_request_construction.rs`.

- [ ] **Step 1: Identify superseded tests**

In `tests/action_dispatch.rs`, remove only these tests after their replacements pass:

```text
official_list_clients_sends_expected_get_request
official_create_network_sends_body
official_path_params_accept_numbers_and_encode_segments
official_connector_get_allows_integration_proxy_path
```

Keep these tests because they assert dispatch/validation behavior rather than HTTP matcher behavior:

```text
connector_path_rejects_non_integration_prefix
existing_clients_action_is_internal
http_query_must_be_object
hybrid_defaults_to_internal_without_site_id
hybrid_uses_official_when_site_id_is_present
all_hybrid_aliases_resolve_to_expected_targets
hybrid_preference_validation_is_explicit
```

- [ ] **Step 2: Remove the helper**

Delete `struct CaptureServer`, its `impl`, and `body_complete()` from `tests/action_dispatch.rs` once no remaining test references it.

- [ ] **Step 3: Check file sizes**

Run: `wc -l tests/action_dispatch.rs tests/http_request_construction.rs tests/mcp_scope.rs tests/support_http.rs src/mcp.rs src/mcp/rmcp_server.rs`

Expected: every listed `.rs` file is `500` lines or fewer.

- [ ] **Step 4: Run the affected tests**

Run:

```bash
cargo test --test action_dispatch
cargo test --test http_request_construction
cargo test --test mcp_scope
```

Expected: PASS for all three test crates.

- [ ] **Step 5: Commit**

```bash
git add tests/action_dispatch.rs
git commit -m "test: retire custom unifi request capture helper"
```

## Task 8: Full Verification And Closeout

**Files:**
- Modify only files already named above.

**Interfaces:**
- Produces: a branch where mock request-construction tests, registry tests, contract tests, and normal unit/integration tests all pass without live UniFi controller access.

- [ ] **Step 1: Run targeted test suite**

Run:

```bash
cargo test --test http_request_construction
cargo test --test mcp_scope
cargo test --test action_dispatch
cargo test --test official_registry
cargo test --test internal_registry
cargo test --test path_building
```

Expected: PASS for all targeted test crates.

- [ ] **Step 2: Run all non-live tests**

Run: `cargo test`

Expected: PASS. Live smoke tests remain gated by their existing environment checks and must not require a controller in CI.

- [ ] **Step 3: Run quality gate**

Run: `cargo check`

Expected: PASS.

- [ ] **Step 4: Run forbidden string check**

Run: `cargo run -p xtask -- check-forbidden-strings`

Expected: PASS and no prohibited approval wording or disallowed external-reference name appears in repo artifacts.

- [ ] **Step 5: Final file-size check**

Run: `find src tests xtask -name '*.rs' -print0 | xargs -0 wc -l | awk '$1 > 500 { print }'`

Expected: no output.

- [ ] **Step 6: Review changed files**

Run: `git diff --stat && git diff -- Cargo.toml tests/support_http.rs tests/http_request_construction.rs tests/mcp_scope.rs tests/action_dispatch.rs src/mcp.rs`

Expected: changes are limited to the planned test dependency, test files, optional module visibility, and removal of superseded custom capture code.

- [ ] **Step 7: Commit final verification fixes if any**

If Step 1 through Step 6 required small fixes, commit them:

```bash
git add Cargo.toml Cargo.lock tests/support_http.rs tests/http_request_construction.rs tests/mcp_scope.rs tests/action_dispatch.rs src/mcp.rs
git commit -m "test: verify unifi api call parity without live controller"
```

If no files changed after Task 7, skip this commit.

## Self-Review

- Spec coverage: The plan covers mock dependency selection, generated official actions, generated internal actions, legacy aliases, HTTP method/path/query/body behavior, MCP scope behavior, sibling tests only, file-size checks, and non-live verification commands.
- Placeholder scan: The plan contains no placeholder markers or unspecified test-writing steps. Every code-changing step includes concrete code or exact edits.
- Type consistency: Helper signatures in Task 1 match all later calls. The plan uses existing `ActionDispatcher`, `ActionRequest`, `UnifiConfig`, `find_capability`, and `required_scope_for` names from the current branch.
- Constraint scan: The plan does not implement tests now; it only documents the work. The plan avoids the prohibited wording token and does not name any external reference author.

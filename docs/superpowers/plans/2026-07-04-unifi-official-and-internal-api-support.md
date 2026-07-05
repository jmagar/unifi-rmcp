# UniFi Official And Internal API Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add complete support for the official UniFi Network API while retaining and expanding support for deeper internal UniFi Network capabilities that are not available in the official API.

**Architecture:** Build two explicit upstream API families: `OfficialNetworkApi` for documented `/proxy/network/integration/v1/...` endpoints and `InternalNetworkApi` for legacy/internal `/proxy/network/api/s/{site}/...` plus `/proxy/network/v2/api/site/{site}/...` endpoints. Expose both through one action-dispatched MCP/CLI surface with capability metadata that marks each action as `official`, `internal`, or `hybrid`.

**Tech Stack:** Rust 2024, `reqwest`, `serde`, `serde_json`, `schemars`, `clap` if CLI growth warrants it, existing rmcp-style single-tool dispatch, `bd` for issue tracking.

## Global Constraints

- Preserve current read-only actions and response behavior unless a new action is explicitly introduced.
- Do not remove internal endpoints; official API coverage is additive.
- Official Network API base for UniFi OS consoles is `/proxy/network/integration/v1`.
- Internal V1 Network API base for UniFi OS consoles is `/proxy/network/api/s/{site}`.
- Internal V2 Network API base for UniFi OS consoles is `/proxy/network/v2/api/site/{site}`.
- Every action must declare source family: `official`, `internal`, or `hybrid`.
- Default to safe read-only behavior; mutating actions require explicit confirmation parameters.
- Use generated registries for high-cardinality endpoint metadata; do not hand-maintain 78 official endpoints or 180 internal-compatible actions in scattered match arms.
- Use `xtask` for repo-local codegen and API inventory refresh tasks; do not add one-off Python scripts for maintained build/dev workflows.
- Keep tests in sibling integration test files under `tests/`; do not add inline `#[cfg(test)] mod tests` blocks to source files.
- No Rust source file may exceed 500 lines of code. Split files before crossing the limit.
- Do not create hyphenated filenames. Use snake_case for new Rust, data, docs, and xtask files.
- Do not create `mod.rs` files. Use modern Rust module roots such as `src/api.rs` with child modules in `src/api/`.
- Keep clean boundaries: HTTP transport, path construction, capability registry, action dispatch, MCP schema, and CLI parsing live in separate modules.
- Do not introduce custom macros for the initial implementation. Use generated Rust source or typed data structures; allow ordinary derive macros such as `Serialize`, `Deserialize`, and `JsonSchema`.
- Use `CLAUDE.md` as the repo memory source of truth; keep `AGENTS.md` and `GEMINI.md` symlinked if memory files are changed.
- Use `bd` for tracking non-trivial implementation work before code changes.

---

## Target File Structure

- Modify `src/unifi.rs`: keep current `UnifiClient` as the low-level HTTP client or split it after Task 1.
- Create `src/api.rs`: common API family exports.
- Create `src/api/http.rs`: shared request execution, error normalization, JSON parsing, API key header handling.
- Create `src/api/official.rs`: official Network Integration API path helpers and typed request entrypoints.
- Create `src/api/internal.rs`: internal V1/V2 path helpers and typed request entrypoints.
- Create `src/capabilities.rs`: action registry types, capability metadata, source-family enum.
- Create `src/capabilities/official_network.rs`: generated official endpoint registry.
- Create `src/capabilities/internal_network.rs`: curated internal capability action registry.
- Create `src/actions.rs`: action dispatch boundary used by MCP and CLI.
- Create `src/actions/official.rs`: official endpoint handlers.
- Create `src/actions/internal.rs`: internal endpoint handlers.
- Modify `src/app.rs`: route service calls through `actions` instead of one method per legacy action.
- Modify `src/mcp/schemas.rs`: generate MCP schema from registry.
- Modify `src/mcp/tools.rs`: dispatch via registry/action layer.
- Modify `src/cli.rs`: dispatch through the same registry/action layer.
- Create `xtask/Cargo.toml`: repo-local automation crate.
- Create `xtask/src/main.rs`: xtask command routing.
- Create `xtask/src/official_api.rs`: official OpenAPI inventory refresh logic.
- Create `xtask/src/internal_reference.rs`: internal capability reference inventory refresh logic.
- Create `data/unifi_official_network_v10_3_58.json`: checked-in official Network API operation inventory.
- Create `data/unifi_internal_reference_tools.json`: checked-in normalized internal capability reference inventory.
- Create `tests/official_registry.rs`: official endpoint registry coverage tests.
- Create `tests/internal_registry.rs`: internal action registry coverage tests.
- Create `tests/path_building.rs`: official/internal path construction tests.
- Create `tests/action_dispatch.rs`: unified action dispatch tests.
- Create `tests/mcp_schema.rs`: MCP schema contains generated actions and confirmation requirements.
- Create `docs/unifi_api_coverage.md`: human-readable coverage matrix.

---

## Task 1: Track Work And Freeze Inventories

**Files:**
- Create: `data/unifi_official_network_v10_3_58.json`
- Create: `data/unifi_internal_reference_tools.json`
- Create: `xtask/Cargo.toml`
- Create: `xtask/src/main.rs`
- Create: `xtask/src/official_api.rs`
- Create: `xtask/src/internal_reference.rs`
- Create: `docs/unifi_api_coverage.md`

**Interfaces:**
- Produces: stable inventory files consumed by registry-generation tasks.
- Produces: `docs/unifi_api_coverage.md` as the reviewed source for scope decisions.

- [ ] **Step 1: Create a bead for the implementation program**

Run:

```bash
bd create --title="Add official UniFi API and internal capability coverage" --description="Implement complete official Network Integration API support while preserving and expanding internal UniFi Network actions." --type=epic --priority=2
bd update <created-id> --claim
```

Expected: one claimed epic exists before code changes.

- [ ] **Step 2: Create the xtask crate**

Create `xtask/Cargo.toml`:

```toml
[package]
name = "xtask"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
anyhow = "1"
regex = "1"
reqwest = { version = "0.12", features = ["blocking", "json", "rustls-tls"], default-features = false }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

Create `xtask/src/main.rs`:

```rust
mod official_api;
mod internal_reference;

use anyhow::{bail, Result};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("refresh-official-api") => official_api::refresh(),
        Some("refresh-internal-reference") => internal_reference::refresh(),
        Some(other) => bail!("unknown xtask command: {other}"),
        None => bail!("usage: cargo run -p xtask -- <refresh-official-api|refresh-internal-reference>"),
    }
}
```

- [ ] **Step 3: Implement official API inventory refresh**

Create `xtask/src/official_api.rs`. The command must first try likely raw OpenAPI document URLs from `developer.ui.com`. If none return JSON with `openapi` and `paths`, fall back to extracting the OpenAPI-shaped operation payload from the public docs pages.

```rust
use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::Serialize;

const BASE: &str = "https://developer.ui.com";
const SERVICE: &str = "network";
const VERSION: &str = "v10.3.58";
const SEED: &str = "https://developer.ui.com/network/v10.3.58/getnetworksoverviewpage";
const OUTPUT: &str = "data/unifi_official_network_v10_3_58.json";

#[derive(Debug, Serialize)]
struct Operation {
    method: String,
    path: String,
    operation_id: String,
    summary: String,
    doc_url: String,
}

#[derive(Debug, Serialize)]
struct Inventory {
    service: &'static str,
    version: &'static str,
    source: &'static str,
    count: usize,
    operations: Vec<Operation>,
}

pub fn refresh() -> Result<()> {
    let operations = fetch_from_raw_openapi().or_else(|_| fetch_from_docs_payload())?;
    if operations.len() != 78 {
        bail!("expected 78 official Network operations for {VERSION}, got {}", operations.len());
    }

    let inventory = Inventory {
        service: SERVICE,
        version: VERSION,
        source: SEED,
        count: operations.len(),
        operations,
    };

    let body = serde_json::to_string_pretty(&inventory)?;
    std::fs::create_dir_all("data")?;
    std::fs::write(OUTPUT, format!("{body}\n"))?;
    Ok(())
}

fn fetch_from_raw_openapi() -> Result<Vec<Operation>> {
    let candidates = [
        "https://developer.ui.com/network/v10.3.58/openapi.json",
        "https://developer.ui.com/network/v10.3.58/openapi",
        "https://developer.ui.com/api/network/v10.3.58/openapi.json",
    ];

    for url in candidates {
        let response = reqwest::blocking::get(url).with_context(|| format!("GET {url} failed"))?;
        if !response.status().is_success() {
            continue;
        }
        let value: serde_json::Value = response.json().with_context(|| format!("GET {url} was not JSON"))?;
        if value.get("openapi").is_some() && value.get("paths").is_some() {
            return operations_from_openapi_json(url, &value);
        }
    }

    bail!("no public raw OpenAPI JSON endpoint found")
}

fn operations_from_openapi_json(source_url: &str, value: &serde_json::Value) -> Result<Vec<Operation>> {
    let paths = value
        .get("paths")
        .and_then(|paths| paths.as_object())
        .context("OpenAPI JSON missing paths object")?;
    let mut operations = Vec::new();
    for (path, methods) in paths {
        let Some(methods) = methods.as_object() else {
            continue;
        };
        for (method, operation) in methods {
            let method_upper = method.to_ascii_uppercase();
            if !matches!(method_upper.as_str(), "GET" | "POST" | "PUT" | "PATCH" | "DELETE") {
                continue;
            }
            operations.push(Operation {
                method: method_upper,
                path: path.clone(),
                operation_id: operation
                    .get("operationId")
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    .to_string(),
                summary: operation
                    .get("summary")
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    .to_string(),
                doc_url: source_url.to_string(),
            });
        }
    }
    operations.sort_by(|left, right| (&left.path, &left.method, &left.operation_id).cmp(&(&right.path, &right.method, &right.operation_id)));
    Ok(operations)
}

fn fetch_from_docs_payload() -> Result<Vec<Operation>> {
    let seed_html = reqwest::blocking::get(SEED)?.text()?;
    let doc_re = Regex::new(r#"\\"path\\":\\"(/network/v10\.3\.58/[^\\"]+)\\",\\"method\\":\\"[A-Z]+\\""#)?;
    let op_re = Regex::new(
        r#"\\"(?P<path>/(?:v1|ea)/[^\\"]+)\\",\\"method\\":\\"(?P<method>[A-Z]+)\\",\\"operationId\\":\\"(?P<operation_id>[^\\"]+)\\",\\"summary\\":\\"(?P<summary>[^\\"]+)"#,
    )?;

    let mut doc_paths = doc_re
        .captures_iter(&seed_html)
        .map(|caps| caps[1].to_string())
        .collect::<Vec<_>>();
    doc_paths.sort();
    doc_paths.dedup();

    let mut operations = Vec::new();
    for doc_path in doc_paths {
        let doc_url = format!("{BASE}{doc_path}");
        let html = reqwest::blocking::get(&doc_url)?.text()?;
        let caps = op_re
            .captures(&html)
            .with_context(|| format!("no operation found in {doc_url}"))?;
        operations.push(Operation {
            method: caps["method"].to_string(),
            path: caps["path"].to_string(),
            operation_id: caps["operation_id"].to_string(),
            summary: caps["summary"].to_string(),
            doc_url,
        });
    }

    operations.sort_by(|left, right| (&left.path, &left.method, &left.operation_id).cmp(&(&right.path, &right.method, &right.operation_id)));
    Ok(operations)
}
```

- [ ] **Step 4: Generate the official API inventory**

Run:

```bash
cargo run -p xtask -- refresh-official-api
```

Expected: JSON contains `"count": 78`.

- [ ] **Step 5: Capture the internal capability reference inventory**

Create `xtask/src/internal_reference.rs`. The output file must not name the upstream project in its filename or user-facing docs; it is an implementation research snapshot for internal capability parity.

Run:

```bash
cargo run -p xtask -- refresh-internal-reference
```

Expected: JSON contains `"count": 180`.

- [ ] **Step 6: Write the initial coverage document**

Create `docs/unifi_api_coverage.md` with sections:

```markdown
# UniFi API Coverage

## Sources

- Official Network API: `data/unifi_official_network_v10_3_58.json`
- Internal capability reference inventory: `data/unifi_internal_reference_tools.json`

## API Families

- `official`: documented Network Integration API under `/proxy/network/integration/v1`.
- `internal`: undocumented Network controller APIs under `/proxy/network/api/s/{site}` and `/proxy/network/v2/api/site/{site}`.
- `hybrid`: action uses internal routes by default and selects official API when `siteId` or `prefer="official"` is supplied.

## Initial Coverage

- Official Network operations targeted: 78.
- Internal Network reference rows captured: 180; verified runtime capabilities: 16.
- Existing rustifi actions preserved: clients, devices, wlans, health, alarms, events, sysinfo, me.
```

- [ ] **Step 7: Commit inventory work**

Run:

```bash
git add xtask data/unifi_official_network_v10_3_58.json data/unifi_internal_reference_tools.json docs/unifi_api_coverage.md
git commit -m "docs: capture unifi api coverage inventories"
```

Expected: commit succeeds.

---

## Task 2: Split HTTP And Path Building Into Official/Internal Clients

**Files:**
- Create: `src/api.rs`
- Create: `src/api/http.rs`
- Create: `src/api/official.rs`
- Create: `src/api/internal.rs`
- Modify: `src/lib.rs`
- Modify: `src/unifi.rs`
- Test: `tests/path_building.rs`

**Interfaces:**
- Produces: `ApiSourceFamily`.
- Produces: `OfficialNetworkApi::path(path: &str) -> String`.
- Produces: `InternalNetworkApi::v1_site_path(suffix: &str) -> String`.
- Produces: `InternalNetworkApi::v2_site_path(suffix: &str) -> String`.

- [ ] **Step 1: Write path-building tests**

Create `tests/path_building.rs`:

```rust
use rustifi::api::internal::InternalNetworkApi;
use rustifi::api::official::OfficialNetworkApi;

#[test]
fn official_network_paths_use_integration_prefix() {
    let api = OfficialNetworkApi::new_for_test("https://gateway.local");
    assert_eq!(
        api.path("/v1/sites"),
        "/proxy/network/integration/v1/sites"
    );
    assert_eq!(
        api.path("v1/sites/site-1/clients"),
        "/proxy/network/integration/v1/sites/site-1/clients"
    );
}

#[test]
fn internal_v1_paths_use_site_prefix() {
    let api = InternalNetworkApi::new_for_test("https://gateway.local", "default", false);
    assert_eq!(
        api.v1_site_path("stat/sta"),
        "/proxy/network/api/s/default/stat/sta"
    );
}

#[test]
fn internal_v2_paths_use_site_prefix() {
    let api = InternalNetworkApi::new_for_test("https://gateway.local", "default", false);
    assert_eq!(
        api.v2_site_path("firewall-policies"),
        "/proxy/network/v2/api/site/default/firewall-policies"
    );
}

#[test]
fn legacy_internal_v1_paths_skip_proxy_prefix() {
    let api = InternalNetworkApi::new_for_test("https://legacy.local:8443", "default", true);
    assert_eq!(api.v1_site_path("stat/sta"), "/api/s/default/stat/sta");
}
```

- [ ] **Step 2: Run the failing test**

Run:

```bash
cargo test --test path_building
```

Expected: FAIL because `rustifi::api` does not exist.

- [ ] **Step 3: Implement API modules**

Create `src/api.rs`:

```rust
pub mod http;
pub mod internal;
pub mod official;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiSourceFamily {
    Official,
    Internal,
    Hybrid,
}
```

Create `src/api/official.rs`:

```rust
#[derive(Debug, Clone)]
pub struct OfficialNetworkApi {
    base_url: String,
}

impl OfficialNetworkApi {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
        }
    }

    pub fn new_for_test(base_url: impl Into<String>) -> Self {
        Self::new(base_url)
    }

    pub fn path(&self, path: &str) -> String {
        let normalized = path.trim_start_matches('/');
        if let Some(rest) = normalized.strip_prefix("v1/") {
            format!("/proxy/network/integration/v1/{rest}")
        } else {
            format!("/proxy/network/integration/{normalized}")
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, self.path(path))
    }
}
```

Create `src/api/internal.rs`:

```rust
#[derive(Debug, Clone)]
pub struct InternalNetworkApi {
    base_url: String,
    site: String,
    legacy: bool,
}

impl InternalNetworkApi {
    pub fn new(base_url: impl Into<String>, site: impl Into<String>, legacy: bool) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            site: site.into(),
            legacy,
        }
    }

    pub fn new_for_test(base_url: impl Into<String>, site: impl Into<String>, legacy: bool) -> Self {
        Self::new(base_url, site, legacy)
    }

    pub fn v1_site_path(&self, suffix: &str) -> String {
        let suffix = suffix.trim_start_matches('/');
        let prefix = if self.legacy { "" } else { "/proxy/network" };
        format!("{prefix}/api/s/{site}/{suffix}", site = self.site)
    }

    pub fn v2_site_path(&self, suffix: &str) -> String {
        let suffix = suffix.trim_start_matches('/');
        if self.legacy {
            format!("/v2/api/site/{site}/{suffix}", site = self.site)
        } else {
            format!("/proxy/network/v2/api/site/{site}/{suffix}", site = self.site)
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}
```

Create `src/api/http.rs` by moving the shared `reqwest` execution logic out of `src/unifi.rs` without changing behavior.

- [ ] **Step 4: Export the module**

Modify `src/lib.rs`:

```rust
pub mod api;
```

Keep existing exports intact.

- [ ] **Step 5: Run path tests**

Run:

```bash
cargo test --test path_building
```

Expected: PASS.

- [ ] **Step 6: Run existing tests**

Run:

```bash
cargo test
```

Expected: PASS.

- [ ] **Step 7: Commit API split**

Run:

```bash
git add src/api src/lib.rs src/unifi.rs tests/path_building.rs
git commit -m "refactor: split unifi api path builders"
```

Expected: commit succeeds.

---

## Task 3: Add Capability Registry

**Files:**
- Create: `src/capabilities.rs`
- Create: `src/capabilities/official_network.rs`
- Create: `src/capabilities/internal_network.rs`
- Modify: `src/lib.rs`
- Test: `tests/official_registry.rs`
- Test: `tests/internal_registry.rs`

**Interfaces:**
- Produces: `Capability`.
- Produces: `all_capabilities() -> &'static [Capability]`.
- Produces: `find_capability(action: &str) -> Option<&'static Capability>`.

- [ ] **Step 1: Write registry tests**

Create `tests/official_registry.rs`:

```rust
use rustifi::api::ApiSourceFamily;
use rustifi::capabilities::{all_capabilities, find_capability};

#[test]
fn official_registry_contains_all_network_operations() {
    let official: Vec<_> = all_capabilities()
        .iter()
        .filter(|cap| cap.source == ApiSourceFamily::Official)
        .collect();
    assert_eq!(official.len(), 78);
}

#[test]
fn official_clients_operation_is_registered() {
    let cap = find_capability("official_list_clients").expect("official clients action");
    assert_eq!(cap.method, Some("GET"));
    assert_eq!(cap.path, Some("/v1/sites/{siteId}/clients"));
    assert_eq!(cap.source, ApiSourceFamily::Official);
}
```

Create `tests/internal_registry.rs`:

```rust
use rustifi::api::ApiSourceFamily;
use rustifi::capabilities::find_capability;

#[test]
fn existing_internal_actions_are_registered() {
    for action in ["clients", "devices", "wlans", "health", "alarms", "events", "sysinfo", "me"] {
        let cap = find_capability(action).unwrap_or_else(|| panic!("missing {action}"));
        assert_eq!(cap.source, ApiSourceFamily::Internal);
    }
}

#[test]
fn internal_gap_examples_are_registered() {
    for action in [
        "internal_list_alarms",
        "internal_list_events",
        "internal_get_network_health",
        "internal_list_port_forwards",
        "internal_list_dns_records",
        "internal_get_switch_ports",
        "internal_trigger_rf_scan",
    ] {
        let cap = find_capability(action).unwrap_or_else(|| panic!("missing {action}"));
        assert_eq!(cap.source, ApiSourceFamily::Internal);
    }
}
```

- [ ] **Step 2: Run failing registry tests**

Run:

```bash
cargo test --test official_registry --test internal_registry
```

Expected: FAIL because `capabilities` does not exist.

- [ ] **Step 3: Implement registry types**

Create `src/capabilities.rs`:

```rust
use crate::api::ApiSourceFamily;

pub mod internal_network;
pub mod official_network;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capability {
    pub action: &'static str,
    pub title: &'static str,
    pub source: ApiSourceFamily,
    pub method: Option<&'static str>,
    pub path: Option<&'static str>,
    pub mutating: bool,
    pub requires_confirmation: bool,
}

pub fn all_capabilities() -> &'static [Capability] {
    static ALL: std::sync::OnceLock<Vec<Capability>> = std::sync::OnceLock::new();
    ALL.get_or_init(|| {
        let mut caps = Vec::new();
        caps.extend_from_slice(official_network::CAPABILITIES);
        caps.extend_from_slice(internal_network::CAPABILITIES);
        caps
    })
}

pub fn find_capability(action: &str) -> Option<&'static Capability> {
    all_capabilities().iter().find(|cap| cap.action == action)
}
```

- [ ] **Step 4: Implement official registry**

Generate `src/capabilities/official_network.rs` from `data/unifi_official_network_v10_3_58.json`. Each action name should be deterministic:

```rust
use crate::api::ApiSourceFamily;
use crate::capabilities::Capability;

pub const CAPABILITIES: &[Capability] = &[
    Capability {
        action: "official_list_clients",
        title: "List Connected Clients",
        source: ApiSourceFamily::Official,
        method: Some("GET"),
        path: Some("/v1/sites/{siteId}/clients"),
        mutating: false,
        requires_confirmation: false,
    },
    Capability {
        action: "official_create_network",
        title: "Create Network",
        source: ApiSourceFamily::Official,
        method: Some("POST"),
        path: Some("/v1/sites/{siteId}/networks"),
        mutating: true,
        requires_confirmation: true,
    },
];
```

The generated file must contain all 78 operations. Mutating methods are `POST`, `PUT`, `PATCH`, and `DELETE`, except read-only connector passthrough `GET`.

- [ ] **Step 5: Implement internal registry**

Create `src/capabilities/internal_network.rs` with existing actions plus the internal target surface grouped by capability. Start with registry metadata only; handlers are added in later tasks.

- [ ] **Step 6: Export capabilities**

Modify `src/lib.rs`:

```rust
pub mod capabilities;
```

- [ ] **Step 7: Run registry tests**

Run:

```bash
cargo test --test official_registry --test internal_registry
```

Expected: PASS.

- [ ] **Step 8: Commit registry**

Run:

```bash
git add src/capabilities src/lib.rs tests/official_registry.rs tests/internal_registry.rs
git commit -m "feat: add unifi capability registry"
```

Expected: commit succeeds.

---

## Task 4: Implement Official Endpoint Dispatcher

**Files:**
- Create: `src/actions.rs`
- Create: `src/actions/official.rs`
- Modify: `src/app.rs`
- Modify: `src/lib.rs`
- Test: `tests/action_dispatch.rs`

**Interfaces:**
- Produces: `ActionRequest { action, params, confirm }`.
- Produces: `ActionDispatcher::execute(request) -> Result<Value>`.
- Consumes: `Capability` registry.

- [ ] **Step 1: Write dispatch tests for official reads and confirmation**

Create `tests/action_dispatch.rs`:

```rust
use serde_json::json;

use rustifi::actions::{ActionDispatcher, ActionRequest};
use rustifi::config::UnifiConfig;

fn test_config() -> UnifiConfig {
    UnifiConfig {
        url: "https://gateway.local".into(),
        api_key: "test-key".into(),
        site: "default".into(),
        skip_tls_verify: true,
        legacy: false,
    }
}

#[tokio::test]
async fn mutating_official_action_requires_confirmation() {
    let dispatcher = ActionDispatcher::new_for_test(test_config());
    let result = dispatcher
        .execute(ActionRequest {
            action: "official_create_network".into(),
            params: json!({"siteId": "site-1", "body": {"name": "IoT"}}),
            confirm: false,
        })
        .await;
    let message = result.unwrap_err().to_string();
    assert!(message.contains("requires confirmation"));
}
```

- [ ] **Step 2: Run failing dispatch test**

Run:

```bash
cargo test --test action_dispatch
```

Expected: FAIL because `actions` does not exist.

- [ ] **Step 3: Implement action request and confirmation gate**

Create `src/actions.rs`:

```rust
pub mod official;
pub mod internal;

use anyhow::{bail, Result};
use serde_json::Value;

use crate::capabilities::find_capability;
use crate::config::UnifiConfig;

#[derive(Debug, Clone)]
pub struct ActionRequest {
    pub action: String,
    pub params: Value,
    pub confirm: bool,
}

pub struct ActionDispatcher {
    cfg: UnifiConfig,
}

impl ActionDispatcher {
    pub fn new(cfg: UnifiConfig) -> Self {
        Self { cfg }
    }

    pub fn new_for_test(cfg: UnifiConfig) -> Self {
        Self::new(cfg)
    }

    pub async fn execute(&self, request: ActionRequest) -> Result<Value> {
        let Some(capability) = find_capability(&request.action) else {
            bail!("unknown UniFi action: {}", request.action);
        };
        if capability.requires_confirmation && !request.confirm {
            bail!("action {} requires confirmation", capability.action);
        }
        official::execute(&self.cfg, capability, &request.params).await
    }
}
```

- [ ] **Step 4: Implement official generic request execution**

Create `src/actions/official.rs` with generic path-template substitution:

```rust
use anyhow::{bail, Result};
use serde_json::Value;

use crate::api::ApiSourceFamily;
use crate::capabilities::Capability;
use crate::config::UnifiConfig;

pub async fn execute(_cfg: &UnifiConfig, capability: &Capability, params: &Value) -> Result<Value> {
    if capability.source != ApiSourceFamily::Official {
        bail!("{} is not an official API action", capability.action);
    }
    let Some(path_template) = capability.path else {
        bail!("official action {} has no path", capability.action);
    };
    let _path = substitute_path(path_template, params)?;
    Ok(serde_json::json!({
        "dry_run": true,
        "method": capability.method,
        "path": _path
    }))
}

fn substitute_path(template: &str, params: &Value) -> Result<String> {
    let mut path = template.to_string();
    for key in ["siteId", "networkId", "clientId", "deviceId", "portIdx"] {
        let needle = format!("{{{key}}}");
        if path.contains(&needle) {
            let Some(value) = params.get(key).and_then(|value| value.as_str()) else {
                bail!("missing required path parameter: {key}");
            };
            path = path.replace(&needle, value);
        }
    }
    Ok(path)
}
```

- [ ] **Step 5: Export actions**

Modify `src/lib.rs`:

```rust
pub mod actions;
```

- [ ] **Step 6: Run dispatch tests**

Run:

```bash
cargo test --test action_dispatch
```

Expected: PASS for confirmation behavior.

- [ ] **Step 7: Replace dry-run with real HTTP after tests are added**

Add tests for request method/path using a local mock HTTP server, then replace `dry_run` with `reqwest` execution through `src/api/http.rs`. The real execution must:

- Substitute all path parameters.
- Append query parameters from `params.query`.
- Send JSON body from `params.body`.
- Add `X-API-Key`.
- Parse JSON response.
- Return useful errors on 401, 403, 404, and non-JSON bodies.

- [ ] **Step 8: Commit official dispatcher**

Run:

```bash
git add src/actions src/app.rs src/lib.rs tests/action_dispatch.rs
git commit -m "feat: dispatch official unifi api actions"
```

Expected: commit succeeds.

---

## Task 5: Preserve Existing Actions As Internal Handlers

**Files:**
- Create: `src/actions/internal.rs`
- Modify: `src/actions.rs`
- Modify: `src/app.rs`
- Modify: `src/unifi.rs`
- Test: existing tests plus `tests/action_dispatch.rs`

**Interfaces:**
- Consumes: existing `UnifiClient` behavior.
- Produces: internal action dispatch for current actions.

- [ ] **Step 1: Add tests for current action compatibility**

Extend `tests/action_dispatch.rs`:

```rust
#[tokio::test]
async fn existing_clients_action_is_internal() {
    let dispatcher = ActionDispatcher::new_for_test(test_config());
    let cap = rustifi::capabilities::find_capability("clients").expect("clients capability");
    assert_eq!(cap.path, Some("/stat/sta"));
}
```

- [ ] **Step 2: Implement internal dispatcher**

Create `src/actions/internal.rs`:

```rust
use anyhow::{bail, Result};
use serde_json::Value;

use crate::api::ApiSourceFamily;
use crate::capabilities::Capability;
use crate::config::UnifiConfig;
use crate::unifi::UnifiClient;

pub async fn execute(cfg: &UnifiConfig, capability: &Capability, params: &Value) -> Result<Value> {
    if capability.source != ApiSourceFamily::Internal {
        bail!("{} is not an internal API action", capability.action);
    }

    let client = UnifiClient::new(cfg)?;
    match capability.action {
        "clients" => client.clients().await,
        "devices" => client.devices().await,
        "wlans" => client.wlans().await,
        "health" => client.health().await,
        "alarms" => client.alarms().await,
        "events" => {
            let mut value = client.events().await?;
            if let Some(limit) = params.get("limit").and_then(|value| value.as_u64()) {
                if let Some(items) = value.get_mut("data").and_then(|value| value.as_array_mut()) {
                    items.truncate(limit as usize);
                }
            }
            Ok(value)
        }
        "sysinfo" => client.sysinfo().await,
        "me" => client.me().await,
        other => bail!("internal action {other} is registered but has no handler"),
    }
}
```

- [ ] **Step 3: Route by source family**

Modify `src/actions.rs` so `ActionDispatcher::execute` chooses `official::execute` or `internal::execute` based on `capability.source`.

- [ ] **Step 4: Run all current tests**

Run:

```bash
cargo test
```

Expected: PASS.

- [ ] **Step 5: Commit compatibility layer**

Run:

```bash
git add src/actions src/app.rs src/unifi.rs tests/action_dispatch.rs
git commit -m "refactor: route existing unifi actions through dispatcher"
```

Expected: commit succeeds.

---

## Task 6: Generate MCP Schema And CLI From Registry

**Files:**
- Modify: `src/mcp/schemas.rs`
- Modify: `src/mcp/tools.rs`
- Modify: `src/cli.rs`
- Modify: `src/main.rs`
- Test: `tests/mcp_schema.rs`
- Test: `tests/cli_parse.rs`
- Test: `tests/tool_dispatch.rs`

**Interfaces:**
- Consumes: `all_capabilities()`.
- Produces: MCP input schema action enum from registry.
- Produces: CLI action lookup from registry.

- [ ] **Step 1: Add MCP schema tests**

Create `tests/mcp_schema.rs`:

```rust
use rustifi::mcp::schemas::tool_definitions;

#[test]
fn schema_contains_official_and_internal_actions() {
    let tools = tool_definitions();
    let schema = &tools[0]["inputSchema"]["properties"]["action"]["enum"];
    let actions = schema.as_array().expect("action enum");

    assert!(actions.iter().any(|value| value == "clients"));
    assert!(actions.iter().any(|value| value == "official_list_clients"));
    assert!(actions.iter().any(|value| value == "internal_list_alarms"));
}

#[test]
fn schema_exposes_confirmation_parameter() {
    let tools = tool_definitions();
    assert!(tools[0]["inputSchema"]["properties"].get("confirm").is_some());
}
```

- [ ] **Step 2: Run failing MCP schema tests**

Run:

```bash
cargo test --test mcp_schema
```

Expected: FAIL because schema is still static.

- [ ] **Step 3: Generate schema enum from capabilities**

Modify `src/mcp/schemas.rs` so `tool_definitions()` builds the action enum from `all_capabilities()`. Include common parameters:

```json
{
  "action": "string enum",
  "params": "object",
  "confirm": "boolean",
  "limit": "integer legacy convenience"
}
```

- [ ] **Step 4: Dispatch MCP calls through `ActionDispatcher`**

Modify `src/mcp/tools.rs` so it converts MCP arguments into:

```rust
ActionRequest {
    action,
    params,
    confirm,
}
```

Then call `ActionDispatcher::execute`.

- [ ] **Step 5: Update CLI parsing**

Modify `src/cli.rs` to accept:

```bash
unifi official_list_clients --param siteId=<uuid> --json
unifi official_create_network --param siteId=<uuid> --body-json '{"name":"IoT"}' --confirm
unifi clients --json
```

Keep the old subcommands working.

- [ ] **Step 6: Run CLI and MCP tests**

Run:

```bash
cargo test --test mcp_schema --test cli_parse --test tool_dispatch
```

Expected: PASS.

- [ ] **Step 7: Commit generated schema/CLI**

Run:

```bash
git add src/mcp src/cli.rs src/main.rs tests/mcp_schema.rs tests/cli_parse.rs tests/tool_dispatch.rs
git commit -m "feat: generate unifi tool schema from capability registry"
```

Expected: commit succeeds.

---

## Task 7: Implement All Official Network API Operations

**Files:**
- Modify: `src/actions/official.rs`
- Modify: `src/capabilities/official_network.rs`
- Test: `tests/official_registry.rs`
- Test: `tests/action_dispatch.rs`

**Interfaces:**
- Consumes: official registry.
- Produces: working HTTP execution for all 78 official operations.

- [ ] **Step 1: Add coverage test that every official operation is dispatchable**

Extend `tests/official_registry.rs`:

```rust
use rustifi::api::ApiSourceFamily;
use rustifi::capabilities::all_capabilities;

#[test]
fn every_official_operation_has_method_and_path() {
    for cap in all_capabilities().iter().filter(|cap| cap.source == ApiSourceFamily::Official) {
        assert!(cap.method.is_some(), "{} missing method", cap.action);
        assert!(cap.path.is_some(), "{} missing path", cap.action);
    }
}
```

- [ ] **Step 2: Implement complete path parameter substitution**

Support every official path variable:

```text
id
path
siteId
aclRuleId
clientId
deviceId
portIdx
dnsPolicyId
firewallPolicyId
firewallZoneId
voucherId
networkId
lagId
mcLagDomainId
switchStackId
trafficMatchingListId
wifiBroadcastId
```

- [ ] **Step 3: Implement connector path safety**

For `official_connector_*` actions, reject `path` values that do not begin with:

```text
/proxy/network/integration/
/proxy/protect/integration/
```

Return an error containing `connector path is outside the supported integration API prefix`.

- [ ] **Step 4: Implement query/body handling**

`params.query` maps to URL query parameters. `params.body` maps to JSON request body. Path variables are read from top-level `params`.

- [ ] **Step 5: Implement response handling**

Return parsed JSON for JSON responses. For empty successful responses, return:

```json
{"success": true}
```

- [ ] **Step 6: Run official dispatch tests**

Run:

```bash
cargo test --test official_registry --test action_dispatch
```

Expected: PASS.

- [ ] **Step 7: Commit official coverage**

Run:

```bash
git add src/actions/official.rs src/capabilities/official_network.rs tests/official_registry.rs tests/action_dispatch.rs
git commit -m "feat: support official unifi network api operations"
```

Expected: commit succeeds.

---

## Task 8: Implement Internal Capability Coverage In Batches

**Files:**
- Modify: `src/capabilities/internal_network.rs`
- Modify: `src/actions/internal.rs`
- Modify: `src/api/internal.rs`
- Test: `tests/internal_registry.rs`
- Test: `tests/action_dispatch.rs`
- Update: `docs/unifi_api_coverage.md`

**Interfaces:**
- Consumes: `data/unifi_internal_reference_tools.json`.
- Produces: internal-compatible action handlers.

Implement in these sub-batches, each with tests and a commit:

1. **Read-only inventory and details**
   - clients, devices, networks, WLANs, vouchers, firewall zones/policies, ACL rules, DNS records, port profiles, user groups, routes, VPN, traffic routes.
2. **Health, telemetry, and events**
   - dashboard, health, stats, alarms, events, alerts, anomalies, IPS, speedtest, traffic flows, DPI traffic.
3. **Gateway and site settings**
   - gateway settings, site settings, SNMP, auto-backup settings, backups.
4. **Switch/AP controls**
   - switch ports, port stats, LLDP, port profile assignment, PoE cycle, aggregation, mirroring, STP, RF scan, radio config, rogue APs, AP groups.
5. **Client/device mutations**
   - block/unblock, rename, forget, fixed IP/local DNS, reconnect, reboot, locate, LEDs, provision, upgrade, toggle.
6. **Policy/config CRUD**
   - port forwards, firewall groups, content filters, OON policies, QoS rules, static routes, usergroups, WLANs.
7. **MCP helpers**
   - batch, batch status, tool index, execute, load tools, subscribe events.

For each sub-batch:

- [ ] **Step 1: Add registry assertions for the batch**

Add exact action names to `tests/internal_registry.rs`.

- [ ] **Step 2: Add handler tests for representative paths**

Use mock HTTP responses to verify method, path, confirmation gate, and body shape.

- [ ] **Step 3: Implement the minimum handlers**

Map each action to an internal V1 or V2 endpoint. Preserve stable action aliases where practical.

- [ ] **Step 4: Run targeted tests**

Run:

```bash
cargo test --test internal_registry --test action_dispatch
```

Expected: PASS.

- [ ] **Step 5: Update coverage docs**

Update `docs/unifi_api_coverage.md` with:

```markdown
| Action | Family | Endpoint | Status |
|---|---|---|---|
| internal_list_alarms | internal | GET /stat/alarm | implemented |
```

- [ ] **Step 6: Commit the batch**

Run:

```bash
git add src/capabilities/internal_network.rs src/actions/internal.rs src/api/internal.rs tests/internal_registry.rs tests/action_dispatch.rs docs/unifi_api_coverage.md
git commit -m "feat: add <batch-name> unifi internal actions"
```

Expected: commit succeeds.

---

## Task 9: Add Hybrid Convenience Actions

**Files:**
- Modify: `src/capabilities/internal_network.rs`
- Modify: `src/actions.rs`
- Create: `src/actions/hybrid.rs`
- Test: `tests/action_dispatch.rs`
- Update: `docs/unifi_api_coverage.md`

**Interfaces:**
- Produces: stable user-facing actions that choose official first and internal fallback when needed.

- [ ] **Step 1: Define hybrid policy**

Create actions:

```text
list_clients
list_devices
list_networks
list_wifi
get_system_info
```

Each action uses official API by default and accepts:

```json
{"prefer": "official" | "internal"}
```

- [ ] **Step 2: Implement hybrid fallback rules**

Fallback to internal only when:

- Official endpoint returns 404 for a capability gap.
- Official endpoint omits fields required by the legacy formatter.
- User explicitly passes `"prefer": "internal"`.

- [ ] **Step 3: Add tests**

Test that `list_clients` chooses official when available and internal when `prefer=internal`.

- [ ] **Step 4: Commit hybrid layer**

Run:

```bash
git add src/actions/hybrid.rs src/actions.rs src/capabilities/internal_network.rs tests/action_dispatch.rs docs/unifi_api_coverage.md
git commit -m "feat: add hybrid unifi convenience actions"
```

Expected: commit succeeds.

---

## Task 10: Live Smoke Testing Against Cloud Gateway Max

**Files:**
- Create: `tests/live_official_smoke.rs`
- Create: `tests/live_internal_smoke.rs`
- Update: `.env.example`
- Update: `README.md`

**Interfaces:**
- Consumes environment:
  - `UNIFI_URL`
  - `UNIFI_API_KEY`
  - `UNIFI_SITE_ID`
  - `UNIFI_SITE`
  - `UNIFI_SKIP_TLS_VERIFY`

- [ ] **Step 1: Add ignored live tests**

Create live tests marked `#[ignore]` that call:

```text
official_get_info
official_list_sites
official_list_clients
official_list_devices
clients
devices
health
events
```

- [ ] **Step 2: Document live test command**

Add to README:

```bash
UNIFI_URL=https://<gateway> \
UNIFI_API_KEY=<network integration key> \
UNIFI_SITE_ID=<uuid> \
UNIFI_SITE=default \
UNIFI_SKIP_TLS_VERIFY=true \
cargo test --test live_official_smoke -- --ignored
```

- [ ] **Step 3: Run normal tests**

Run:

```bash
cargo test
```

Expected: PASS without live controller.

- [ ] **Step 4: Run live tests**

Run the README command on the local network.

Expected: official and internal smoke tests pass, or failures are documented as controller-version gaps.

- [ ] **Step 5: Commit live smoke harness**

Run:

```bash
git add tests/live_official_smoke.rs tests/live_internal_smoke.rs .env.example README.md
git commit -m "test: add live unifi api smoke tests"
```

Expected: commit succeeds.

---

## Task 11: Closeout, Docs, And Release

**Files:**
- Update: `README.md`
- Update: `plugins/unifi/skills/unifi/SKILL.md`
- Update: `plugins/unifi/.claude-plugin/plugin.json`
- Update: `docs/unifi_api_coverage.md`

**Interfaces:**
- Produces: user-facing docs for official/internal/hybrid action families.

- [ ] **Step 1: Update user docs**

README must explain:

- Official API actions use `/proxy/network/integration/v1`.
- Internal actions use `/proxy/network/api/s/{site}` and `/proxy/network/v2/api/site/{site}`.
- Mutating actions require `confirm=true`.
- Hybrid actions prefer official API unless configured otherwise.

- [ ] **Step 2: Update skill docs**

Update the UniFi skill action reference to include:

```text
official_* actions
internal_* actions
hybrid convenience actions
```

- [ ] **Step 3: Run quality gates**

Run:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Expected: all pass.

- [ ] **Step 4: Push beads and git**

Run:

```bash
git pull --rebase
bd dolt push
git push
git status
```

Expected: status reports branch is up to date with origin.

- [ ] **Step 5: Commit docs closeout**

Run:

```bash
git add README.md plugins/unifi/skills/unifi/SKILL.md plugins/unifi/.claude-plugin/plugin.json docs/unifi_api_coverage.md
git commit -m "docs: document official and internal unifi api support"
```

Expected: commit succeeds.

---

## Self-Review

- Spec coverage: The plan covers all official Network API operations through a generated registry and generic official dispatcher, and covers internal support through a normalized reference inventory and batched handler implementation.
- Risk: Exact internal endpoint mapping for all 180 reference tools must be verified batch-by-batch from source code evidence; do not infer paths from tool names when implementing.
- Risk: Official `siteId` is a UUID, while current internal `site` is usually `default`; both must remain separate config values.
- Risk: Connector proxy routes are official but dangerous because they proxy arbitrary paths; the plan restricts them to integration API prefixes.
- Test strategy: registry count tests prevent accidental endpoint loss; mock HTTP tests prove path/method/body construction; ignored live tests verify real Cloud Gateway behavior.

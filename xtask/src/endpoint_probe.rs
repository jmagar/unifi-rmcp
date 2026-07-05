use std::{collections::BTreeMap, path::Path};

use anyhow::{Context, Result, bail};
use reqwest::blocking::Client;
use rustifi::api::{internal::InternalNetworkApi, official::OfficialNetworkApi, path};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const CONNECTOR_PREFIXES: &[&str] = &["/proxy/network/integration/", "/proxy/protect/integration/"];

#[derive(Debug, Deserialize)]
pub struct OfficialInventory {
    pub operations: Vec<OfficialOperation>,
}

#[derive(Debug, Deserialize)]
pub struct OfficialOperation {
    pub method: String,
    pub path: String,
    pub operation_id: String,
    pub summary: String,
}

#[derive(Debug, Deserialize)]
pub struct InternalInventory {
    pub tools: Vec<InternalTool>,
}

#[derive(Debug, Deserialize)]
pub struct InternalTool {
    pub action: String,
    pub method: String,
    pub path: String,
    pub title: String,
    pub mutating: bool,
    pub verified: bool,
    #[serde(default)]
    pub runtime: bool,
    #[serde(default)]
    pub verification_mode: Option<String>,
    #[serde(default)]
    pub auth_scope: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub generated_at: String,
    pub mode: String,
    pub totals: Totals,
    pub results: Vec<ProbeResult>,
}

#[derive(Debug, Default, Serialize)]
pub struct Totals {
    pub total: usize,
    pub ok: usize,
    pub requires_fixture: usize,
    pub unsupported: usize,
    pub rejected: usize,
    pub auth_failed: usize,
    pub server_error: usize,
    pub skipped: usize,
}

#[derive(Debug, Serialize)]
pub struct ProbeResult {
    pub family: &'static str,
    pub name: String,
    pub method: String,
    pub template: String,
    pub path: Option<String>,
    pub mutating: bool,
    pub verified_reference: Option<bool>,
    pub status: ProbeStatus,
    pub http_status: Option<u16>,
    pub verdict: String,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeStatus {
    LiveOk,
    ContractOk,
    RequiresFixture,
    Unsupported,
    Rejected,
    AuthFailed,
    ServerError,
    Skipped,
    BudgetExhausted,
    ContractError,
}

impl ProbeStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LiveOk => "live_ok",
            Self::ContractOk => "contract_ok",
            Self::RequiresFixture => "requires_fixture",
            Self::Unsupported => "unsupported",
            Self::Rejected => "rejected",
            Self::AuthFailed => "auth_failed",
            Self::ServerError => "server_error",
            Self::Skipped => "skipped",
            Self::BudgetExhausted => "budget_exhausted",
            Self::ContractError => "contract_error",
        }
    }
}

pub struct Config {
    pub base_url: String,
    pub api_key: String,
    pub site: String,
    pub site_id: Option<String>,
    pub skip_tls_verify: bool,
    pub legacy: bool,
    pub verify_unverified_internal: bool,
    pub max_requests: usize,
    pub timeout_secs: u64,
    pub rate_limit_ms: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let base_url = resolved_base_url(&env_required("UNIFI_URL")?)?;
        let api_key = env_required("UNIFI_API_KEY")?;
        let site = std::env::var("UNIFI_SITE").unwrap_or_else(|_| "default".to_string());
        let site_id = std::env::var("UNIFI_SITE_ID")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let skip_tls_verify = env_bool(
            "UNIFI_SKIP_TLS_VERIFY",
            env_bool("UNIFI_ALLOW_INSECURE_TLS", true),
        );
        let legacy = env_bool("UNIFI_LEGACY", false);
        let verify_unverified_internal = env_bool("UNIFI_VERIFY_UNVERIFIED_INTERNAL", false);
        let max_requests = env_usize("UNIFI_VERIFY_MAX_REQUESTS", 200);
        let timeout_secs = env_u64("UNIFI_VERIFY_TIMEOUT_SECS", 12);
        let rate_limit_ms = env_u64("UNIFI_VERIFY_RATE_LIMIT_MS", 0);

        Ok(Self {
            base_url,
            api_key,
            site,
            site_id,
            skip_tls_verify,
            legacy,
            verify_unverified_internal,
            max_requests,
            timeout_secs,
            rate_limit_ms,
        })
    }
}

fn resolved_base_url(raw_url: &str) -> Result<String> {
    let mut url = reqwest::Url::parse(raw_url.trim()).context("UNIFI_URL must be a URL")?;
    if let Ok(resolve_ip) = std::env::var("UNIFI_RESOLVE_IP") {
        if !resolve_ip.trim().is_empty() {
            url.set_host(Some(resolve_ip.trim()))
                .map_err(|_| anyhow::anyhow!("UNIFI_RESOLVE_IP is not a valid host"))?;
        }
    }
    Ok(url.as_str().trim_end_matches('/').to_string())
}

pub fn official_path(cfg: &Config, template: &str) -> Option<String> {
    let params = official_path_params(cfg, template)?;
    let substituted = path::substitute_path(template, &params, CONNECTOR_PREFIXES).ok()?;
    Some(OfficialNetworkApi::new_for_test("").path(&substituted))
}

fn official_path_params(cfg: &Config, template: &str) -> Option<Value> {
    let mut params = serde_json::Map::new();
    if template.contains("{siteId}") {
        params.insert(
            "siteId".to_string(),
            Value::String(cfg.site_id.as_deref()?.to_string()),
        );
    }
    for key in [
        "id",
        "aclRuleId",
        "clientId",
        "deviceId",
        "dnsPolicyId",
        "firewallPolicyId",
        "firewallZoneId",
        "voucherId",
        "networkId",
        "lagId",
        "mcLagDomainId",
        "switchStackId",
        "trafficMatchingListId",
        "wifiBroadcastId",
    ] {
        if template.contains(&format!("{{{key}}}")) {
            params.insert(
                key.to_string(),
                Value::String("00000000-0000-4000-8000-000000000000".to_string()),
            );
        }
    }
    if template.contains("{portIdx}") {
        params.insert("portIdx".to_string(), Value::String("9999".to_string()));
    }
    if template.contains("*path") {
        params.insert(
            "path".to_string(),
            Value::String("/proxy/network/integration/v1/info".to_string()),
        );
    }
    Some(Value::Object(params))
}

pub fn internal_path(cfg: &Config, template: &str) -> String {
    if template == "/api/self" {
        return if cfg.legacy {
            template.to_string()
        } else {
            "/proxy/network/api/self".to_string()
        };
    }
    let api = InternalNetworkApi::new_for_test("", &cfg.site, cfg.legacy);
    if let Some(suffix) = template.strip_prefix("/v2/") {
        return api.v2_site_path(suffix.trim_start_matches("api/site/{site}/"));
    }
    let suffix = template.trim_start_matches('/');
    api.v1_site_path(suffix)
}

pub fn discover_site_id(client: &Client, cfg: &Config) -> Result<Option<String>> {
    let url = format!(
        "{}/proxy/network/integration/v1/sites",
        cfg.base_url.trim_end_matches('/')
    );
    let response = client
        .get(&url)
        .header("X-API-KEY", &cfg.api_key)
        .header("Accept", "application/json")
        .send()
        .with_context(|| format!("discover site id from {url}"))?;
    if !response.status().is_success() {
        return Ok(None);
    }
    let value: Value = response.json().context("parse official sites response")?;
    Ok(find_site_id(&value))
}

fn find_site_id(value: &Value) -> Option<String> {
    match value {
        Value::Object(object) => {
            for key in ["id", "siteId", "site_id"] {
                if let Some(id) = object.get(key).and_then(Value::as_str) {
                    return Some(id.to_string());
                }
            }
            for key in ["data", "items", "sites", "results"] {
                if let Some(found) = object.get(key).and_then(find_site_id) {
                    return Some(found);
                }
            }
            object.values().find_map(find_site_id)
        }
        Value::Array(items) => items.iter().find_map(find_site_id),
        _ => None,
    }
}

pub fn inert_body(name: &str, title: &str) -> Value {
    json!({
        "_rustifi_probe": true,
        "name": format!("rustifi-endpoint-verification-{name}"),
        "title": title,
        "cmd": "rustifi_endpoint_probe",
        "action": "rustifi_endpoint_probe"
    })
}

pub fn classify_status(status: u16) -> (&'static str, ProbeStatus) {
    match status {
        200..=299 => ("live_ok", ProbeStatus::LiveOk),
        400 | 404 | 405 | 409 | 422 => ("route_reached_rejected_probe", ProbeStatus::Rejected),
        401 | 403 => ("auth_or_permission_failed", ProbeStatus::AuthFailed),
        500..=599 => ("server_error", ProbeStatus::ServerError),
        _ => ("unexpected_status", ProbeStatus::Rejected),
    }
}

pub fn totals(results: &[ProbeResult]) -> Totals {
    let mut totals = Totals {
        total: results.len(),
        ..Default::default()
    };
    for result in results {
        match result.status {
            ProbeStatus::LiveOk | ProbeStatus::ContractOk => totals.ok += 1,
            ProbeStatus::RequiresFixture => totals.requires_fixture += 1,
            ProbeStatus::Unsupported => totals.unsupported += 1,
            ProbeStatus::Rejected | ProbeStatus::ContractError => totals.rejected += 1,
            ProbeStatus::AuthFailed => totals.auth_failed += 1,
            ProbeStatus::ServerError => totals.server_error += 1,
            ProbeStatus::Skipped | ProbeStatus::BudgetExhausted => totals.skipped += 1,
        }
    }
    totals
}

pub fn skipped(
    family: &'static str,
    name: &str,
    method: &str,
    template: &str,
    mutating: bool,
) -> ProbeResult {
    ProbeResult {
        family,
        name: name.to_string(),
        method: method.to_string(),
        template: template.to_string(),
        path: None,
        mutating,
        verified_reference: None,
        status: ProbeStatus::Skipped,
        http_status: None,
        verdict: "disabled_by_mode".to_string(),
        detail: String::new(),
    }
}

pub fn budget_exhausted(
    family: &'static str,
    name: &str,
    method: &str,
    template: &str,
    mutating: bool,
) -> ProbeResult {
    ProbeResult {
        family,
        name: name.to_string(),
        method: method.to_string(),
        template: template.to_string(),
        path: None,
        mutating,
        verified_reference: None,
        status: ProbeStatus::BudgetExhausted,
        http_status: None,
        verdict: "live_budget_exhausted".to_string(),
        detail: "increase UNIFI_VERIFY_MAX_REQUESTS or use contract mode".to_string(),
    }
}

pub fn detail(text: String) -> String {
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    compact.chars().take(1024).collect()
}

pub fn timestamp() -> String {
    let now = time::OffsetDateTime::now_utc();
    now.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "unknown".to_string())
}

fn env_required(name: &str) -> Result<String> {
    let value = std::env::var(name).with_context(|| format!("{name} is required"))?;
    if value.trim().is_empty() {
        bail!("{name} is required");
    }
    Ok(value)
}

fn env_bool(name: &str, default: bool) -> bool {
    std::env::var(name)
        .ok()
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(default)
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}

pub fn load_dotenv(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(());
    }
    let body = std::fs::read_to_string(path)?;
    let existing = std::env::vars().collect::<BTreeMap<_, _>>();
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if existing.contains_key(key) {
            continue;
        }
        let value = value
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();
        unsafe {
            std::env::set_var(key.trim(), value);
        }
    }
    Ok(())
}

use anyhow::{Result, bail};
use rustifi::api::ApiSourceFamily;
use rustifi::capabilities::{AuthScope, Capability, find_capability, official_network};

use crate::endpoint_probe::{Config, InternalTool, OfficialOperation, Report, official_path};

pub(crate) struct LiveBudget {
    remaining: usize,
}

impl LiveBudget {
    pub(crate) fn new(max_requests: usize) -> Self {
        Self {
            remaining: max_requests,
        }
    }

    pub(crate) fn try_take(&mut self) -> bool {
        if self.remaining == 0 {
            return false;
        }
        self.remaining -= 1;
        true
    }
}

pub(crate) fn take_live_budget(budget: Option<&mut LiveBudget>) -> bool {
    budget.is_none_or(LiveBudget::try_take)
}

pub(crate) fn official_contract_status(mutating: bool, path: &str) -> &'static str {
    if !mutating && !path.contains("*path") && requires_fixture(path) {
        "requires_fixture"
    } else {
        "contract_ok"
    }
}

pub(crate) fn official_contract_status_for(op: &OfficialOperation, mutating: bool) -> &'static str {
    let action = official_network::action_name(&op.operation_id);
    let Some(capability) = find_capability(&action) else {
        return "contract_error";
    };
    if !official_contract_valid(capability, op, mutating) {
        return "contract_error";
    }
    official_contract_status(mutating, &op.path)
}

pub(crate) fn requires_fixture(path: &str) -> bool {
    let needs_path_fixture = path.contains('{') && path.replace("{siteId}", "").contains('{');
    let needs_query_fixture = matches!(path, "/v1/sites/{siteId}/firewall/policies/ordering");
    needs_path_fixture || needs_query_fixture
}

pub(crate) fn internal_contract_valid(tool: &InternalTool) -> bool {
    let expected = match tool.auth_scope.as_deref().unwrap_or("read") {
        "read" => AuthScope::Read,
        "admin" => AuthScope::Admin,
        _ => return false,
    };
    if tool.runtime {
        let Some(capability) = find_capability(&tool.action) else {
            return false;
        };
        capability.source == ApiSourceFamily::Internal
            && capability.method.as_deref() == Some(tool.method.as_str())
            && capability.path.as_deref() == Some(tool.path.as_str())
            && capability.mutating == tool.mutating
            && capability.auth_scope == expected
    } else {
        find_capability(&tool.action).is_none()
    }
}

pub(crate) fn fail_on_bad_status(report: &Report, contract_mode: bool, output: &str) -> Result<()> {
    if report.totals.rejected > 0 || report.totals.auth_failed > 0 || report.totals.server_error > 0
    {
        bail!("endpoint verification failed; see {output}");
    }
    if contract_mode && report.totals.skipped > 0 {
        bail!(
            "contract endpoint verification skipped {} endpoints; see {output}",
            report.totals.skipped
        );
    }
    Ok(())
}

fn official_contract_valid(
    capability: &Capability,
    op: &OfficialOperation,
    mutating: bool,
) -> bool {
    capability.source == ApiSourceFamily::Official
        && capability.method.as_deref() == Some(op.method.as_str())
        && capability.path.as_deref() == Some(op.path.as_str())
        && capability.mutating == mutating
        && capability.auth_scope == expected_scope(mutating)
        && official_path(&contract_config(), &op.path).is_some()
}

fn contract_config() -> Config {
    Config {
        base_url: "https://unifi.example.invalid".to_string(),
        api_key: "contract".to_string(),
        site: "default".to_string(),
        site_id: Some("00000000-0000-4000-8000-000000000000".to_string()),
        skip_tls_verify: false,
        legacy: false,
        verify_unverified_internal: false,
        max_requests: 0,
        timeout_secs: 0,
        rate_limit_ms: 0,
    }
}

fn expected_scope(mutating: bool) -> AuthScope {
    if mutating {
        AuthScope::Admin
    } else {
        AuthScope::Read
    }
}

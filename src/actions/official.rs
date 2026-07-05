use anyhow::{bail, Context, Result};
use reqwest::Method;
use serde_json::Value;

use crate::api::{http, official::OfficialNetworkApi, ApiSourceFamily};
use crate::capabilities::Capability;
use crate::config::UnifiConfig;

pub async fn execute(cfg: &UnifiConfig, capability: &Capability, params: &Value) -> Result<Value> {
    if capability.source != ApiSourceFamily::Official {
        bail!("{} is not an official API action", capability.action);
    }
    let path_template = capability
        .path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("official action {} has no path", capability.action))?;
    let method = capability
        .method
        .as_deref()
        .unwrap_or("GET")
        .parse::<Method>()
        .context("invalid official HTTP method")?;
    let path = substitute_path(&capability.action, path_template, params)?;
    let api = OfficialNetworkApi::new(&cfg.url);
    let full_path = api.path(&path);
    http::request_json(
        cfg,
        method,
        &full_path,
        params.get("query"),
        params.get("body"),
    )
    .await
}

fn substitute_path(action: &str, template: &str, params: &Value) -> Result<String> {
    let mut path = template.to_string();
    for key in [
        "id",
        "siteId",
        "aclRuleId",
        "clientId",
        "deviceId",
        "portIdx",
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
        let needle = format!("{{{key}}}");
        if path.contains(&needle) {
            let Some(value) = params.get(key).and_then(Value::as_str) else {
                bail!("missing required path parameter: {key}");
            };
            path = path.replace(&needle, value);
        }
    }
    if path.contains("*path") {
        let Some(value) = params.get("path").and_then(Value::as_str) else {
            bail!("missing required path parameter: path");
        };
        validate_connector_path(action, value)?;
        path = path.replace("*path", value.trim_start_matches('/'));
    }
    Ok(path)
}

fn validate_connector_path(action: &str, path: &str) -> Result<()> {
    if !action.starts_with("official_connector_") {
        return Ok(());
    }
    if path.starts_with("/proxy/network/integration/")
        || path.starts_with("/proxy/protect/integration/")
    {
        Ok(())
    } else {
        bail!("connector path is outside the supported integration API prefix")
    }
}

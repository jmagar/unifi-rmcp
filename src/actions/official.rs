use anyhow::{bail, Context, Result};
use reqwest::Method;
use serde_json::{Number, Value};

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
            let value = path_scalar(action, params, key)?;
            path = path.replace(&needle, &encode_path_segment(&value));
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

fn path_scalar(action: &str, params: &Value, key: &str) -> Result<String> {
    match params.get(key) {
        Some(Value::String(value)) => Ok(value.clone()),
        Some(Value::Number(value)) => Ok(number_to_string(value)),
        Some(Value::Bool(value)) => Ok(value.to_string()),
        Some(_) => bail!("{action} path parameter {key} must be a string, number, or boolean"),
        None => bail!("missing required path parameter: {key}"),
    }
}

fn number_to_string(value: &Number) -> String {
    value
        .as_i64()
        .map(|v| v.to_string())
        .or_else(|| value.as_u64().map(|v| v.to_string()))
        .or_else(|| value.as_f64().map(|v| v.to_string()))
        .unwrap_or_else(|| value.to_string())
}

fn encode_path_segment(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char);
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
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

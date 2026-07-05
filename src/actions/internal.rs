use anyhow::{bail, Context, Result};
use reqwest::Method;
use serde_json::Value;

use crate::api::{http, internal::InternalNetworkApi, path, ApiSourceFamily};
use crate::capabilities::Capability;
use crate::config::UnifiConfig;
use crate::unifi::UnifiClient;

pub async fn execute(cfg: &UnifiConfig, capability: &Capability, params: &Value) -> Result<Value> {
    if capability.source != ApiSourceFamily::Internal {
        bail!("{} is not an internal API action", capability.action);
    }

    let client = UnifiClient::new(cfg)?;
    match capability.action.as_str() {
        "clients" => client.clients().await,
        "devices" => client.devices().await,
        "wlans" => client.wlans().await,
        "health" => client.health().await,
        "alarms" => client.alarms().await,
        "events" => {
            let mut events = client.events().await?;
            truncate_data_array(
                &mut events,
                params
                    .get("limit")
                    .and_then(Value::as_u64)
                    .map(|value| value as usize),
            );
            Ok(events)
        }
        "sysinfo" => client.sysinfo().await,
        "me" => client.me().await,
        _ => execute_generic(cfg, capability, params).await,
    }
}

async fn execute_generic(
    cfg: &UnifiConfig,
    capability: &Capability,
    params: &Value,
) -> Result<Value> {
    let method = capability
        .method
        .as_deref()
        .unwrap_or("GET")
        .parse::<Method>()
        .context("invalid internal HTTP method")?;
    let path = capability
        .path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("internal action {} has no path", capability.action))?;
    let path = path::substitute_path(path, params, &[])?;
    let api = InternalNetworkApi::new(&cfg.url, &cfg.site, cfg.legacy);
    let full_path = if path == "/api/self" {
        if cfg.legacy {
            path
        } else {
            "/proxy/network/api/self".to_string()
        }
    } else if path.starts_with("/api/") {
        if cfg.legacy {
            path
        } else {
            format!("/proxy/network{path}")
        }
    } else if let Some(suffix) = path.strip_prefix("/v2/") {
        api.v2_site_path(suffix.trim_start_matches("api/site/{site}/"))
    } else {
        api.v1_site_path(&path)
    };
    http::request_json(
        cfg,
        method,
        &full_path,
        params.get("query"),
        params.get("body"),
    )
    .await
}

fn truncate_data_array(value: &mut Value, limit: Option<usize>) {
    let Some(limit) = limit else {
        return;
    };
    if let Some(items) = value.get_mut("data").and_then(Value::as_array_mut) {
        items.truncate(limit);
    }
}

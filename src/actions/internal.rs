use anyhow::{bail, Context, Result};
use reqwest::Method;
use serde_json::Value;

use crate::api::{http, internal::InternalNetworkApi, ApiSourceFamily};
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
            let mut value = client.events().await?;
            if let Some(limit) = params.get("limit").and_then(Value::as_u64) {
                if let Some(items) = value.get_mut("data").and_then(Value::as_array_mut) {
                    items.truncate(limit as usize);
                }
            }
            Ok(value)
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
    let api = InternalNetworkApi::new(&cfg.url, &cfg.site, cfg.legacy);
    let full_path = if path == "/api/self" {
        if cfg.legacy {
            path.to_string()
        } else {
            "/proxy/network/api/self".to_string()
        }
    } else if let Some(suffix) = path.strip_prefix("/v2/") {
        api.v2_site_path(suffix.trim_start_matches("api/site/{site}/"))
    } else {
        api.v1_site_path(path)
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

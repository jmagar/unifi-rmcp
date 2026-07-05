use anyhow::{bail, Context, Result};
use reqwest::Method;
use serde_json::Value;

use crate::api::{http, official::OfficialNetworkApi, path, ApiSourceFamily};
use crate::capabilities::Capability;
use crate::config::UnifiConfig;

const CONNECTOR_PREFIXES: &[&str] = &["/proxy/network/integration/", "/proxy/protect/integration/"];

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
    let path = path::substitute_path(path_template, params, CONNECTOR_PREFIXES)?;
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

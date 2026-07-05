use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::blocking::Client;

use crate::endpoint_probe::{
    Config, InternalInventory, OfficialInventory, ProbeResult, Report, classify_status, detail,
    discover_site_id, inert_body, internal_path, load_dotenv, official_path, skipped, timestamp,
    totals,
};

const OFFICIAL_INPUT: &str = "data/unifi_official_network_v10_3_58.json";
const INTERNAL_INPUT: &str = "data/unifi_internal_reference_tools.json";
const OUTPUT: &str = "data/unifi_endpoint_verification_report.json";

pub fn verify() -> Result<()> {
    load_dotenv(".env")?;
    load_dotenv("/home/jmagar/.unifi/.env")?;
    load_dotenv("/home/jmagar/.rustifi/.env")?;
    load_dotenv("/home/jmagar/.labby/.env")?;

    let mut cfg = Config::from_env()?;
    let client = Client::builder()
        .danger_accept_invalid_certs(cfg.skip_tls_verify)
        .http1_only()
        .timeout(Duration::from_secs(12))
        .build()
        .context("build HTTP client")?;
    if cfg.site_id.is_none() {
        cfg.site_id = discover_site_id(&client, &cfg)?;
    }

    let mut results = Vec::new();
    results.extend(probe_official(&client, &cfg)?);
    results.extend(probe_internal(&client, &cfg)?);

    let totals = totals(&results);
    let report = Report {
        generated_at: timestamp(),
        mode: if cfg.include_mutating {
            "all-with-inert-mutating-probes".to_string()
        } else {
            "read-only".to_string()
        },
        base_url: cfg.base_url.clone(),
        site: cfg.site.clone(),
        site_id: cfg.site_id.clone(),
        totals,
        results,
    };

    std::fs::create_dir_all("data")?;
    std::fs::write(
        OUTPUT,
        format!("{}\n", serde_json::to_string_pretty(&report)?),
    )?;
    println!(
        "verified {} endpoints: ok={} rejected={} auth_failed={} server_error={} skipped={}",
        report.totals.total,
        report.totals.ok,
        report.totals.rejected,
        report.totals.auth_failed,
        report.totals.server_error,
        report.totals.skipped
    );
    println!("report: {OUTPUT}");
    Ok(())
}

fn probe_official(client: &Client, cfg: &Config) -> Result<Vec<ProbeResult>> {
    let inventory: OfficialInventory =
        serde_json::from_str(&std::fs::read_to_string(OFFICIAL_INPUT)?)?;
    let mut results = Vec::new();
    for op in inventory.operations {
        let mutating = !op.method.eq_ignore_ascii_case("GET");
        if mutating && !cfg.include_mutating {
            results.push(skipped(
                "official",
                &op.operation_id,
                &op.method,
                &op.path,
                mutating,
            ));
            continue;
        }
        let Some(path) = official_path(cfg, &op.path) else {
            results.push(ProbeResult {
                family: "official",
                name: op.operation_id,
                method: op.method,
                template: op.path,
                path: None,
                mutating,
                verified_reference: None,
                status: "skipped".to_string(),
                http_status: None,
                verdict: "missing_site_id".to_string(),
                detail: "set UNIFI_SITE_ID to probe site-scoped official endpoints".to_string(),
            });
            continue;
        };
        results.push(send_probe(
            client,
            cfg,
            "official",
            op.operation_id,
            op.summary,
            op.method,
            op.path,
            path,
            mutating,
            None,
        ));
    }
    Ok(results)
}

fn probe_internal(client: &Client, cfg: &Config) -> Result<Vec<ProbeResult>> {
    let inventory: InternalInventory =
        serde_json::from_str(&std::fs::read_to_string(INTERNAL_INPUT)?)?;
    let mut results = Vec::new();
    for tool in inventory.tools {
        if !tool.verified && !cfg.verify_unverified_internal {
            results.push(skipped(
                "internal",
                &tool.action,
                &tool.method,
                &tool.path,
                tool.mutating,
            ));
            continue;
        }
        if tool.mutating && !cfg.include_mutating {
            results.push(skipped(
                "internal",
                &tool.action,
                &tool.method,
                &tool.path,
                tool.mutating,
            ));
            continue;
        }
        let path = internal_path(cfg, &tool.path);
        results.push(send_probe(
            client,
            cfg,
            "internal",
            tool.action,
            tool.title,
            tool.method,
            tool.path,
            path,
            tool.mutating,
            Some(tool.verified),
        ));
    }
    Ok(results)
}

#[allow(clippy::too_many_arguments)]
fn send_probe(
    client: &Client,
    cfg: &Config,
    family: &'static str,
    name: String,
    title: String,
    method: String,
    template: String,
    path: String,
    mutating: bool,
    verified_reference: Option<bool>,
) -> ProbeResult {
    let method = method.to_ascii_uppercase();
    let url = format!("{}{}", cfg.base_url, path);
    let mut request = client
        .request(method.parse().unwrap_or(reqwest::Method::GET), &url)
        .header("X-API-KEY", &cfg.api_key)
        .header("Accept", "application/json");
    if mutating {
        request = request.json(&inert_body(&name, &title));
    }

    match request.send() {
        Ok(response) => {
            let status = response.status();
            let status_u16 = status.as_u16();
            let text = response.text().unwrap_or_default();
            let (verdict, status_label) = classify_status(status_u16);
            ProbeResult {
                family,
                name,
                method,
                template,
                path: Some(path),
                mutating,
                verified_reference,
                status: status_label.to_string(),
                http_status: Some(status_u16),
                verdict: verdict.to_string(),
                detail: detail(text),
            }
        }
        Err(error) => ProbeResult {
            family,
            name,
            method,
            template,
            path: Some(path),
            mutating,
            verified_reference,
            status: "server_error".to_string(),
            http_status: None,
            verdict: "request_error".to_string(),
            detail: error.to_string(),
        },
    }
}

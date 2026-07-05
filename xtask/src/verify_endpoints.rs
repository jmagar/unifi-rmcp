use std::{path::PathBuf, time::Duration};

use anyhow::{Context, Result, bail};
use reqwest::blocking::Client;

use crate::endpoint_probe::{
    Config, InternalInventory, OfficialInventory, ProbeResult, Report, classify_status, detail,
    discover_site_id, inert_body, internal_path, load_dotenv, official_path, skipped, timestamp,
    totals,
};

const OFFICIAL_INPUT: &str = "data/unifi_official_network_v10_3_58.json";
const INTERNAL_INPUT: &str = "data/unifi_internal_endpoint_models.json";
const OUTPUT_DIR: &str = "target/unifi_verification";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VerifyMode {
    Contract,
    SafeLive,
    MutatingLive,
}

impl VerifyMode {
    fn parse() -> Result<Self> {
        let args = std::env::args().skip(2).collect::<Vec<_>>();
        if args.iter().any(|arg| arg == "--help" || arg == "-h") {
            print_help();
            std::process::exit(0);
        }
        let mut mode = Self::Contract;
        let mut idx = 0;
        while idx < args.len() {
            match args[idx].as_str() {
                "--mode" => {
                    let Some(value) = args.get(idx + 1) else {
                        bail!("--mode requires contract, safe_live, or mutating_live");
                    };
                    mode = match value.as_str() {
                        "contract" => Self::Contract,
                        "safe_live" => Self::SafeLive,
                        "mutating_live" => Self::MutatingLive,
                        other => bail!("unknown verifier mode {other}"),
                    };
                    idx += 2;
                }
                other => bail!("unknown verify-api-endpoints argument {other}"),
            }
        }
        Ok(mode)
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Contract => "contract",
            Self::SafeLive => "safe_live",
            Self::MutatingLive => "mutating_live",
        }
    }

    fn is_live(self) -> bool {
        !matches!(self, Self::Contract)
    }
}

pub fn verify() -> Result<()> {
    let mode = VerifyMode::parse()?;
    let mut cfg_and_client = None;
    if mode.is_live() {
        load_dotenv(".env")?;
        load_dotenv("/home/jmagar/.unifi/.env")?;
        load_dotenv("/home/jmagar/.rustifi/.env")?;
        load_dotenv("/home/jmagar/.labby/.env")?;

        let mut cfg = Config::from_env()?;
        cfg.include_mutating = mode == VerifyMode::MutatingLive;
        let client = Client::builder()
            .danger_accept_invalid_certs(cfg.skip_tls_verify)
            .http1_only()
            .timeout(Duration::from_secs(cfg.timeout_secs))
            .build()
            .context("build HTTP client")?;
        if cfg.site_id.is_none() {
            cfg.site_id = discover_site_id(&client, &cfg)?;
        }
        cfg_and_client = Some((cfg, client));
    }
    let cfg = cfg_and_client.as_ref().map(|(cfg, _)| cfg);
    let client = cfg_and_client.as_ref().map(|(_, client)| client);

    let mut results = Vec::new();
    results.extend(probe_official(client, cfg, mode)?);
    results.extend(probe_internal(client, cfg, mode)?);

    let totals = totals(&results);
    let report = Report {
        generated_at: timestamp(),
        mode: mode.as_str().to_string(),
        totals,
        results,
    };

    let output = report_path(mode);
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(
        &output,
        format!("{}\n", serde_json::to_string_pretty(&report)?),
    )?;
    println!(
        "verified {} endpoints: live_ok={} contract_ok={} requires_fixture={} rejected={} auth_failed={} server_error={} skipped={}",
        report.totals.total,
        report
            .results
            .iter()
            .filter(|result| result.status == "live_ok")
            .count(),
        report
            .results
            .iter()
            .filter(|result| result.status == "contract_ok")
            .count(),
        report
            .results
            .iter()
            .filter(|result| result.status == "requires_fixture")
            .count(),
        report.totals.rejected,
        report.totals.auth_failed,
        report.totals.server_error,
        report.totals.skipped
    );
    println!("official accounted {}", official_count(&report.results));
    println!("official rejected {}", official_rejected(&report.results));
    println!("report: {}", output.display());
    Ok(())
}

fn print_help() {
    println!(
        "Usage: cargo run -p xtask -- verify-api-endpoints [--mode contract|safe_live|mutating_live]\n\nModes:\n  contract       Validate registry, path, auth-scope, and request policy without network access\n  safe_live      Probe safe read endpoints only; mutating and fixture endpoints stay contract-only\n  mutating_live  Probe all configured endpoints; use only on disposable or controlled sites"
    );
}

fn report_path(mode: VerifyMode) -> PathBuf {
    PathBuf::from(OUTPUT_DIR).join(format!("{}_report.json", mode.as_str()))
}

fn probe_official(
    client: Option<&Client>,
    cfg: Option<&Config>,
    mode: VerifyMode,
) -> Result<Vec<ProbeResult>> {
    let inventory: OfficialInventory =
        serde_json::from_str(&std::fs::read_to_string(OFFICIAL_INPUT)?)?;
    let mut results = Vec::new();
    let mut live_requests = 0;
    for op in inventory.operations {
        let mutating = !op.method.eq_ignore_ascii_case("GET");
        if mode == VerifyMode::Contract || mutating || op.path.contains("*path") {
            let status = official_contract_status(mutating, &op.path);
            results.push(policy_result(
                "official",
                op.operation_id,
                op.method,
                op.path,
                mutating,
                status,
                None,
            ));
            continue;
        }
        if requires_fixture(&op.path) {
            results.push(policy_result(
                "official",
                op.operation_id,
                op.method,
                op.path,
                mutating,
                "requires_fixture",
                None,
            ));
            continue;
        }
        let Some(cfg) = cfg else {
            results.push(policy_result(
                "official",
                op.operation_id,
                op.method,
                op.path,
                mutating,
                "contract_ok",
                None,
            ));
            continue;
        };
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
        let Some(client) = client else {
            continue;
        };
        if live_requests >= cfg.max_requests {
            results.push(skipped(
                "official",
                &op.operation_id,
                &op.method,
                &op.path,
                mutating,
            ));
            continue;
        }
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
        live_requests += 1;
        rate_limit(cfg);
    }
    Ok(results)
}

fn probe_internal(
    client: Option<&Client>,
    cfg: Option<&Config>,
    mode: VerifyMode,
) -> Result<Vec<ProbeResult>> {
    let inventory: InternalInventory =
        serde_json::from_str(&std::fs::read_to_string(INTERNAL_INPUT)?)?;
    let mut results = Vec::new();
    let mut live_requests = 0;
    for tool in inventory.tools {
        let auth_scope = tool.auth_scope.as_deref().unwrap_or("read");
        if mode == VerifyMode::Contract || !tool.runtime {
            let status = if tool.runtime {
                "contract_ok"
            } else {
                tool.verification_mode.as_deref().unwrap_or("unsupported")
            };
            results.push(policy_result(
                "internal",
                tool.action,
                tool.method,
                tool.path,
                tool.mutating,
                status,
                Some(tool.verified),
            ));
            if tool.mutating && auth_scope != "admin" {
                results.last_mut().expect("policy result").status = "unsupported".to_string();
            }
            continue;
        }
        let Some(cfg) = cfg else {
            continue;
        };
        if (!tool.verified && !cfg.verify_unverified_internal)
            || (tool.mutating && mode != VerifyMode::MutatingLive)
        {
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
        let Some(client) = client else {
            continue;
        };
        if live_requests >= cfg.max_requests {
            results.push(skipped(
                "internal",
                &tool.action,
                &tool.method,
                &tool.path,
                tool.mutating,
            ));
            continue;
        }
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
        live_requests += 1;
        rate_limit(cfg);
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

fn policy_result(
    family: &'static str,
    name: String,
    method: String,
    template: String,
    mutating: bool,
    status: &str,
    verified_reference: Option<bool>,
) -> ProbeResult {
    ProbeResult {
        family,
        name,
        method,
        template,
        path: None,
        mutating,
        verified_reference,
        status: status.to_string(),
        http_status: None,
        verdict: status.to_string(),
        detail: String::new(),
    }
}

fn official_contract_status(mutating: bool, path: &str) -> &'static str {
    if path.contains("*path") || mutating {
        "contract_ok"
    } else if requires_fixture(path) {
        "requires_fixture"
    } else {
        "contract_ok"
    }
}

fn requires_fixture(path: &str) -> bool {
    let needs_path_fixture = path.contains('{') && path.replace("{siteId}", "").contains('{');
    let needs_query_fixture = matches!(path, "/v1/sites/{siteId}/firewall/policies/ordering");
    needs_path_fixture || needs_query_fixture
}

fn rate_limit(cfg: &Config) {
    if cfg.rate_limit_ms > 0 {
        std::thread::sleep(Duration::from_millis(cfg.rate_limit_ms));
    }
}

fn official_count(results: &[ProbeResult]) -> usize {
    results
        .iter()
        .filter(|result| result.family == "official")
        .count()
}

fn official_rejected(results: &[ProbeResult]) -> usize {
    results
        .iter()
        .filter(|result| result.family == "official" && result.status == "rejected")
        .count()
}

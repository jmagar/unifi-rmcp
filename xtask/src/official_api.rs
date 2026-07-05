use anyhow::{Context, Result, bail};
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
        bail!(
            "expected 78 official Network operations for {VERSION}, got {}",
            operations.len()
        );
    }

    let inventory = Inventory {
        service: SERVICE,
        version: VERSION,
        source: SEED,
        count: operations.len(),
        operations,
    };

    std::fs::create_dir_all("data")?;
    let body = serde_json::to_string_pretty(&inventory)?;
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
        let value: serde_json::Value = response
            .json()
            .with_context(|| format!("GET {url} was not JSON"))?;
        if value.get("openapi").is_some() && value.get("paths").is_some() {
            return operations_from_openapi_json(url, &value);
        }
    }

    bail!("no public raw OpenAPI JSON endpoint found")
}

fn operations_from_openapi_json(
    source_url: &str,
    value: &serde_json::Value,
) -> Result<Vec<Operation>> {
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
            if !matches!(
                method_upper.as_str(),
                "GET" | "POST" | "PUT" | "PATCH" | "DELETE"
            ) {
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
    operations.sort_by(|left, right| {
        (&left.path, &left.method, &left.operation_id).cmp(&(
            &right.path,
            &right.method,
            &right.operation_id,
        ))
    });
    Ok(operations)
}

fn fetch_from_docs_payload() -> Result<Vec<Operation>> {
    let seed_html = reqwest::blocking::get(SEED)?.text()?;
    let doc_re =
        Regex::new(r#"\\"path\\":\\"(/network/v10\.3\.58/[^\\"]+)\\",\\"method\\":\\"[A-Z]+\\""#)?;
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

    operations.sort_by(|left, right| {
        (&left.path, &left.method, &left.operation_id).cmp(&(
            &right.path,
            &right.method,
            &right.operation_id,
        ))
    });
    Ok(operations)
}

use anyhow::{Result, bail};
use serde_json::Value;

const RAW_OUTPUT: &str = "data/upstream_mcp_network_tools_main.json";
const MODEL_OUTPUT: &str = "data/unifi_internal_endpoint_models.json";

pub fn refresh() -> Result<()> {
    let raw_body = std::fs::read_to_string(RAW_OUTPUT)?;
    let model_body = std::fs::read_to_string(MODEL_OUTPUT)?;
    let raw = read_json(&raw_body)?;
    let model = read_json(&model_body)?;
    validate_inventories(&raw, &model)?;
    write_original(RAW_OUTPUT, &raw_body)?;
    write_original(MODEL_OUTPUT, &model_body)?;
    Ok(())
}

fn read_json(body: &str) -> Result<Value> {
    Ok(serde_json::from_str(body)?)
}

fn write_original(path: &str, body: &str) -> Result<()> {
    let body = body.trim_end_matches('\n');
    std::fs::write(path, format!("{body}\n"))?;
    Ok(())
}

fn validate_inventories(raw: &Value, model: &Value) -> Result<()> {
    let raw_tools = array_len(raw, "tools")?;
    let model_tools = array_len(model, "tools")?;
    let controller_endpoint_count = number(raw, "controller_endpoint_count")?;
    let meta_tool_count = number(raw, "meta_tool_count")?;
    let runtime_count = number(model, "runtime_count")?;
    let non_runtime_count = number(model, "non_runtime_count")?;

    ensure_eq(number(raw, "count")?, raw_tools, "raw count")?;
    ensure_eq(
        controller_endpoint_count + meta_tool_count,
        raw_tools,
        "raw endpoint accounting",
    )?;
    ensure_eq(
        number(model, "source_count")?,
        raw_tools,
        "model source count",
    )?;
    ensure_eq(
        number(model, "accounted_count")?,
        model_tools,
        "model accounted count",
    )?;
    ensure_eq(
        runtime_count + non_runtime_count,
        model_tools,
        "model runtime accounting",
    )?;
    ensure_eq(
        model_tools,
        controller_endpoint_count,
        "controller endpoint model count",
    )?;
    ensure_eq(
        number(model, "meta_tool_count")?,
        meta_tool_count,
        "model meta tool count",
    )?;
    Ok(())
}

fn array_len(value: &Value, key: &str) -> Result<usize> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::len)
        .ok_or_else(|| anyhow::anyhow!("{key} must be an array"))
}

fn number(value: &Value, key: &str) -> Result<usize> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .map(|value| value as usize)
        .ok_or_else(|| anyhow::anyhow!("{key} must be a number"))
}

fn ensure_eq(actual: usize, expected: usize, label: &str) -> Result<()> {
    if actual != expected {
        bail!("{label} mismatch: expected {expected}, got {actual}");
    }
    Ok(())
}

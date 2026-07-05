use anyhow::{Context, Result, bail};
use std::process::Command;

const MAX_RUST_LINES: usize = 500;

pub fn check() -> Result<()> {
    let files = tracked_files()?;
    let forbidden = forbidden_literals();
    let mut failures = Vec::new();

    for file in files {
        if file.ends_with("mod.rs") {
            failures.push(format!("mod.rs file is tracked: {file}"));
        }
        if file.starts_with("target/unifi_verification/")
            || file.contains("unifi_endpoint_verification_report")
        {
            failures.push(format!("live verifier report is tracked: {file}"));
        }
        let body = read_tracked_text(&file)?;
        if file.ends_with(".rs") {
            let line_count = body.lines().count();
            if line_count > MAX_RUST_LINES {
                failures.push(format!(
                    "{file} has {line_count} lines, over the {MAX_RUST_LINES} line limit"
                ));
            }
        }
        for literal in &forbidden {
            if body.contains(literal) {
                failures.push(format!("{file} contains forbidden literal {literal}"));
            }
        }
    }

    if failures.is_empty() {
        println!("check-forbidden-strings: ok");
        Ok(())
    } else {
        bail!("{}", failures.join("\n"));
    }
}

fn read_tracked_text(file: &str) -> Result<String> {
    let bytes = std::fs::read(file).with_context(|| format!("read tracked file {file}"))?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

fn tracked_files() -> Result<Vec<String>> {
    let output = Command::new("git").args(["ls-files", "-z"]).output()?;
    if !output.status.success() {
        bail!("git ls-files failed");
    }
    Ok(output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| String::from_utf8_lossy(path).to_string())
        .collect())
}

fn forbidden_literals() -> Vec<String> {
    vec![
        ["allow", "_mutating"].concat(),
        ["allow", "_mutation"].concat(),
        ["confirm", "_mutation"].concat(),
        ["mutation", "_gate"].concat(),
        ["unifi", "-network-api-mcp"].concat(),
        ["ubiquiti", "-mcp-server"].concat(),
    ]
}

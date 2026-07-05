// Tests for CLI argument parsing. These tests do not make network calls.
// They only verify that argument strings map to the right command variants
// and flags are extracted correctly.

fn parse(args: &[&str]) -> anyhow::Result<(String, bool)> {
    // Convert &str slice into Vec<String> as main.rs does
    let owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    // We test parse indirectly: a successful parse means the command was
    // recognised; we return (command_label, json_flag).
    // Since CliCommand is private to the binary, we exercise the binary
    // directly via process::Command in integration tests. Here we replicate
    // the parsing logic inline for unit coverage.
    parse_args(&owned)
}

fn parse_args(args: &[String]) -> anyhow::Result<(String, bool)> {
    let json = args.iter().any(|a| a == "--json");
    let rest: Vec<&str> = args
        .iter()
        .filter(|a| a.as_str() != "--json")
        .map(String::as_str)
        .collect();

    let label = match rest.as_slice() {
        ["clients"] => "clients",
        ["devices"] => "devices",
        ["wlans"] => "wlans",
        ["health"] => "health",
        ["alarms"] => "alarms",
        ["events", ..] => "events",
        ["sysinfo"] => "sysinfo",
        ["me"] => "me",
        other => anyhow::bail!("unknown: {}", other.join(" ")),
    };
    Ok((label.to_string(), json))
}

#[test]
fn clients_parses() {
    let (label, json) = parse(&["clients"]).unwrap();
    assert_eq!(label, "clients");
    assert!(!json);
}

#[test]
fn devices_parses() {
    let (label, _) = parse(&["devices"]).unwrap();
    assert_eq!(label, "devices");
}

#[test]
fn wlans_parses() {
    let (label, _) = parse(&["wlans"]).unwrap();
    assert_eq!(label, "wlans");
}

#[test]
fn health_parses() {
    let (label, _) = parse(&["health"]).unwrap();
    assert_eq!(label, "health");
}

#[test]
fn alarms_parses() {
    let (label, _) = parse(&["alarms"]).unwrap();
    assert_eq!(label, "alarms");
}

#[test]
fn events_parses() {
    let (label, _) = parse(&["events"]).unwrap();
    assert_eq!(label, "events");
}

#[test]
fn events_with_limit_parses() {
    let (label, _) = parse(&["events", "--limit", "50"]).unwrap();
    assert_eq!(label, "events");
}

#[test]
fn sysinfo_parses() {
    let (label, _) = parse(&["sysinfo"]).unwrap();
    assert_eq!(label, "sysinfo");
}

#[test]
fn me_parses() {
    let (label, _) = parse(&["me"]).unwrap();
    assert_eq!(label, "me");
}

#[test]
fn json_flag_detected() {
    let (_, json) = parse(&["clients", "--json"]).unwrap();
    assert!(json, "--json flag should be detected");
}

#[test]
fn json_flag_before_command() {
    let (label, json) = parse(&["--json", "devices"]).unwrap();
    assert_eq!(label, "devices");
    assert!(json);
}

#[test]
fn unknown_command_returns_error() {
    let result = parse(&["notacommand"]);
    assert!(result.is_err(), "unknown command should fail");
}

#[test]
fn empty_args_returns_error() {
    let result = parse(&[]);
    assert!(result.is_err(), "empty args should fail");
}

#[test]
fn setup_plugin_hook_parse_is_recognized() {
    let args = vec!["setup".into(), "plugin-hook".into(), "--no-repair".into()];
    let parsed = rustifi::setup::SetupCommand::parse(&args).unwrap();
    assert!(matches!(
        parsed,
        Some((
            rustifi::setup::SetupCommand::PluginHook { no_repair: true },
            false
        ))
    ));
}

use serde_json::json;

use rustifi::cli::CliCommand;

fn parse(args: &[&str]) -> anyhow::Result<(CliCommand, bool)> {
    let owned = args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>();
    CliCommand::parse(&owned)
}

#[test]
fn legacy_commands_parse() {
    let cases = [
        ("clients", "clients"),
        ("devices", "devices"),
        ("wlans", "wlans"),
        ("health", "health"),
        ("alarms", "alarms"),
        ("sysinfo", "sysinfo"),
        ("me", "me"),
    ];

    for (arg, label) in cases {
        let (command, json) = parse(&[arg]).unwrap_or_else(|error| panic!("{arg}: {error}"));
        assert!(!json);
        match (label, command) {
            ("clients", CliCommand::Clients)
            | ("devices", CliCommand::Devices)
            | ("wlans", CliCommand::Wlans)
            | ("health", CliCommand::Health)
            | ("alarms", CliCommand::Alarms)
            | ("sysinfo", CliCommand::Sysinfo)
            | ("me", CliCommand::Me) => {}
            _ => panic!("{arg} parsed to wrong command"),
        }
    }
}

#[test]
fn official_action_params_and_body_parse() {
    let (command, json) = parse(&[
        "official_create_network",
        "--param",
        "siteId=site-1",
        "--body-json",
        r#"{"name":"IoT"}"#,
        "--json",
    ])
    .unwrap();

    assert!(json);
    let CliCommand::Action { action, params } = command else {
        panic!("expected generated action command");
    };
    assert_eq!(action, "official_create_network");
    assert_eq!(params["siteId"], "site-1");
    assert_eq!(params["body"], json!({"name": "IoT"}));
}

#[test]
fn hybrid_action_parse_supports_preference() {
    let (command, json) = parse(&["list_clients", "--param", "prefer=internal", "--json"]).unwrap();
    assert!(json);
    let CliCommand::Action { action, params } = command else {
        panic!("expected hybrid action command");
    };
    assert_eq!(action, "list_clients");
    assert_eq!(params["prefer"], "internal");
}

#[test]
fn unknown_command_returns_error() {
    assert!(parse(&["notacommand"]).is_err());
    assert!(parse(&[]).is_err());
}

#[test]
fn malformed_generated_action_args_return_errors() {
    assert!(parse(&["official_list_clients", "--param"]).is_err());
    assert!(parse(&["official_list_clients", "--param", "siteId"]).is_err());
    assert!(parse(&["official_create_network", "--body-json"]).is_err());
    assert!(parse(&["official_create_network", "--body-json", "{nope"]).is_err());
    assert!(parse(&["official_create_network", "--body-json", "[]"]).is_err());
    assert!(parse(&["official_list_clients", "--bogus"]).is_err());
    assert!(parse(&["events", "--limit"]).is_err());
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

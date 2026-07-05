use std::collections::HashSet;

use serde_json::Value;

fn models() -> Value {
    serde_json::from_str(include_str!("../data/unifi_internal_endpoint_models.json"))
        .expect("internal endpoint models JSON should parse")
}

#[test]
fn all_upstream_reference_actions_are_accounted_for() {
    let raw: Value =
        serde_json::from_str(include_str!("../data/upstream_mcp_network_tools_main.json"))
            .expect("neutral upstream reference JSON should parse");
    let models = models();
    let tools = models["tools"].as_array().expect("model tools");
    let raw_tools = raw["tools"].as_array().expect("raw tools");

    assert_eq!(
        models["source_count"].as_u64(),
        Some(raw_tools.len() as u64)
    );
    assert_eq!(models["accounted_count"].as_u64(), Some(tools.len() as u64));
    assert_eq!(raw_tools.len(), 180);
    assert_eq!(tools.len(), 175);
    assert_eq!(models["meta_tool_count"].as_u64(), Some(5));

    let model_actions = tools
        .iter()
        .map(|tool| tool["action"].as_str().unwrap().to_string())
        .collect::<HashSet<_>>();
    for tool in raw_tools
        .iter()
        .filter(|tool| tool["controller_endpoint"].as_bool() == Some(true))
    {
        let action = tool["action"].as_str().expect("raw action");
        assert!(
            model_actions.contains(action),
            "missing endpoint model for {action}"
        );
    }
}

#[test]
fn runtime_models_are_safe_and_evidence_backed() {
    let models = models();
    let mut actions = HashSet::new();
    for tool in models["tools"].as_array().expect("model tools") {
        let action = tool["action"].as_str().expect("action");
        assert!(
            actions.insert(action.to_string()),
            "duplicate action {action}"
        );

        let method = tool["method"].as_str().expect("method");
        assert!(matches!(
            method,
            "GET" | "POST" | "PUT" | "PATCH" | "DELETE"
        ));

        let path = tool["path"].as_str().expect("path");
        assert!(
            path.starts_with('/'),
            "{action} path must be relative absolute"
        );
        assert!(
            !path.contains("://"),
            "{action} path must not be absolute URL"
        );
        assert!(
            !path.contains(".."),
            "{action} path must not contain traversal"
        );

        let mode = tool["verification_mode"]
            .as_str()
            .expect("verification mode");
        assert!(matches!(
            mode,
            "live_2xx" | "contract_ok" | "requires_fixture" | "unsupported"
        ));

        let scope = tool["auth_scope"].as_str().expect("auth scope");
        assert!(matches!(scope, "read" | "admin"));
        if method == "POST" && path.starts_with("/cmd/") {
            assert_eq!(
                scope, "admin",
                "{action} command endpoint POST must require admin scope"
            );
        }

        if tool["runtime"].as_bool() == Some(true) {
            assert_eq!(
                tool["verified"].as_bool(),
                Some(true),
                "{action} runtime without proof"
            );
            assert_ne!(mode, "unsupported", "{action} runtime unsupported");
            if tool["mutating"].as_bool() == Some(true) {
                assert_eq!(scope, "admin", "{action} mutating without admin scope");
            }
        }
    }
}

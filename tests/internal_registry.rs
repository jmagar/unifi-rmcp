use std::collections::HashSet;

use rustifi::api::ApiSourceFamily;
use rustifi::capabilities::{all_capabilities, find_capability};
use serde_json::Value;

#[test]
fn internal_registry_contains_reference_count() {
    let inventory: Value =
        serde_json::from_str(include_str!("../data/unifi_internal_reference_tools.json"))
            .expect("internal reference inventory should parse");
    assert_eq!(inventory["count"].as_u64(), Some(12));
    assert_eq!(
        inventory["tools"].as_array().expect("tools array").len(),
        12
    );

    let internal = all_capabilities()
        .iter()
        .filter(|cap| cap.source == ApiSourceFamily::Internal)
        .collect::<Vec<_>>();
    let verified = inventory["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter(|tool| tool["verified"].as_bool() == Some(true))
        .collect::<Vec<_>>();
    assert_eq!(internal.len(), verified.len());

    let exposed = internal
        .iter()
        .map(|cap| cap.action.as_str())
        .collect::<HashSet<_>>();
    for tool in verified {
        let action = tool["action"].as_str().expect("verified action");
        assert!(exposed.contains(action), "verified {action} is not exposed");
    }
    assert_eq!(internal.len(), 12);
}

#[test]
fn existing_internal_actions_are_registered() {
    for action in [
        "clients", "devices", "wlans", "health", "alarms", "sysinfo", "me",
    ] {
        let cap = find_capability(action).unwrap_or_else(|| panic!("missing {action}"));
        assert_eq!(cap.source, ApiSourceFamily::Internal);
    }
}

#[test]
fn internal_gap_examples_are_registered() {
    for action in [
        "internal_list_alarms",
        "internal_get_network_health",
        "internal_list_port_forwards",
        "internal_trigger_rf_scan",
    ] {
        let cap = find_capability(action).unwrap_or_else(|| panic!("missing {action}"));
        assert_eq!(cap.source, ApiSourceFamily::Internal);
    }
}

#[test]
fn internal_reference_contains_only_verified_rows() {
    let inventory: Value =
        serde_json::from_str(include_str!("../data/unifi_internal_reference_tools.json"))
            .expect("internal reference inventory should parse");
    for tool in inventory["tools"].as_array().expect("tools array") {
        assert_eq!(tool["verified"].as_bool(), Some(true));
    }
    assert!(find_capability("internal_get_networks").is_none());
}

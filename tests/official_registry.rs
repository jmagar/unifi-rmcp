use std::collections::HashSet;

use rustifi::api::ApiSourceFamily;
use rustifi::capabilities::{all_capabilities, find_capability};
use serde_json::Value;

#[test]
fn official_registry_contains_all_network_operations() {
    let inventory: Value =
        serde_json::from_str(include_str!("../data/unifi_official_network_v10_3_58.json"))
            .expect("official inventory should parse");
    let operations = inventory["operations"]
        .as_array()
        .expect("official operations array");
    assert_eq!(inventory["count"].as_u64(), Some(operations.len() as u64));

    let official = all_capabilities()
        .iter()
        .filter(|cap| cap.source == ApiSourceFamily::Official)
        .collect::<Vec<_>>();
    assert_eq!(official.len(), operations.len());
}

#[test]
fn official_clients_operation_is_registered() {
    let cap = find_capability("official_list_clients").expect("official clients action");
    assert_eq!(cap.method.as_deref(), Some("GET"));
    assert_eq!(cap.path.as_deref(), Some("/v1/sites/{siteId}/clients"));
    assert_eq!(cap.source, ApiSourceFamily::Official);
}

#[test]
fn every_official_operation_has_method_and_path() {
    let mut actions = HashSet::new();
    for cap in all_capabilities()
        .iter()
        .filter(|cap| cap.source == ApiSourceFamily::Official)
    {
        assert!(
            actions.insert(cap.action.as_str()),
            "duplicate {}",
            cap.action
        );
        assert!(cap.method.is_some(), "{} missing method", cap.action);
        assert!(cap.path.is_some(), "{} missing path", cap.action);
        assert_eq!(
            cap.mutating,
            cap.method.as_deref() != Some("GET"),
            "{} mutating flag should match method",
            cap.action
        );
    }
}

#[test]
fn official_inventory_count_is_the_parity_floor() {
    let inventory: serde_json::Value =
        serde_json::from_str(include_str!("../data/unifi_official_network_v10_3_58.json"))
            .expect("official inventory should parse");
    assert_eq!(inventory["count"].as_u64(), Some(78));
    assert_eq!(inventory["operations"].as_array().unwrap().len(), 78);
}

#[test]
fn every_official_operation_has_registered_action() {
    let inventory: serde_json::Value =
        serde_json::from_str(include_str!("../data/unifi_official_network_v10_3_58.json"))
            .expect("official inventory should parse");
    for op in inventory["operations"].as_array().unwrap() {
        let operation_id = op["operation_id"].as_str().unwrap();
        let action = rustifi::capabilities::official_network::action_name(operation_id);
        assert!(
            rustifi::capabilities::find_capability(&action).is_some(),
            "missing {action}"
        );
    }
}

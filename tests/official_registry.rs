use rustifi::api::ApiSourceFamily;
use rustifi::capabilities::{all_capabilities, find_capability};

#[test]
fn official_registry_contains_all_network_operations() {
    let official = all_capabilities()
        .iter()
        .filter(|cap| cap.source == ApiSourceFamily::Official)
        .collect::<Vec<_>>();
    assert_eq!(official.len(), 78);
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
    for cap in all_capabilities()
        .iter()
        .filter(|cap| cap.source == ApiSourceFamily::Official)
    {
        assert!(cap.method.is_some(), "{} missing method", cap.action);
        assert!(cap.path.is_some(), "{} missing path", cap.action);
    }
}

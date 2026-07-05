use serde_json::json;

use rustifi::actions::{ActionDispatcher, ActionRequest};
use rustifi::config::UnifiConfig;

fn test_config() -> UnifiConfig {
    UnifiConfig {
        url: "https://gateway.local".into(),
        api_key: "test-key".into(),
        site: "default".into(),
        skip_tls_verify: true,
        legacy: false,
    }
}

#[tokio::test]
async fn mutating_official_action_requires_confirmation() {
    let dispatcher = ActionDispatcher::new_for_test(test_config());
    let result = dispatcher
        .execute(ActionRequest {
            action: "official_create_network".into(),
            params: json!({"siteId": "site-1", "body": {"name": "IoT"}}),
            confirm: false,
        })
        .await;
    let message = result.unwrap_err().to_string();
    assert!(message.contains("requires confirmation"));
}

#[tokio::test]
async fn connector_path_rejects_non_integration_prefix() {
    let dispatcher = ActionDispatcher::new_for_test(test_config());
    let result = dispatcher
        .execute(ActionRequest {
            action: "official_connector_get".into(),
            params: json!({"id": "console-1", "path": "/proxy/network/api/s/default/stat/sta"}),
            confirm: false,
        })
        .await;
    let message = result.unwrap_err().to_string();
    assert!(message.contains("connector path is outside"));
}

#[test]
fn existing_clients_action_is_internal() {
    let cap = rustifi::capabilities::find_capability("clients").expect("clients capability");
    assert_eq!(cap.path.as_deref(), Some("/stat/sta"));
}

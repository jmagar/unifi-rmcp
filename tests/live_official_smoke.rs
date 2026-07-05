use serde_json::json;

use rustifi::actions::{ActionDispatcher, ActionRequest};
use rustifi::config::Config;

#[tokio::test]
#[ignore]
async fn official_smoke_actions() {
    rustifi::config::load_dotenv();
    let config = Config::load().expect("config should load");
    let site_id = std::env::var("UNIFI_SITE_ID").expect("UNIFI_SITE_ID is required");
    let dispatcher = ActionDispatcher::new(config.unifi);
    for (action, params) in [
        ("official_get_info", json!({})),
        ("official_list_sites", json!({})),
        ("official_list_clients", json!({ "siteId": site_id })),
        ("official_list_devices", json!({ "siteId": site_id })),
    ] {
        dispatcher
            .execute(ActionRequest {
                action: action.to_string(),
                params,
            })
            .await
            .unwrap_or_else(|error| panic!("{action} failed: {error}"));
    }
}

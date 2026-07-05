use serde_json::json;

use rustifi::actions::{ActionDispatcher, ActionRequest};
use rustifi::config::Config;

#[tokio::test]
#[ignore]
async fn internal_smoke_actions() {
    rustifi::config::load_dotenv();
    let config = Config::load().expect("config should load");
    let dispatcher = ActionDispatcher::new(config.unifi);
    for action in ["clients", "devices", "health", "events"] {
        dispatcher
            .execute(ActionRequest {
                action: action.to_string(),
                params: json!({}),
                confirm: false,
            })
            .await
            .unwrap_or_else(|error| panic!("{action} failed: {error}"));
    }
}

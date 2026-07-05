use serde_json::json;

use rustifi::actions::{ActionDispatcher, ActionRequest};
use rustifi::config::UnifiConfig;

fn test_config(url: impl Into<String>) -> UnifiConfig {
    UnifiConfig {
        url: url.into(),
        api_key: "test-key".into(),
        site: "default".into(),
        skip_tls_verify: true,
        legacy: false,
    }
}

#[tokio::test]
async fn connector_path_rejects_non_integration_prefix() {
    let dispatcher = ActionDispatcher::new_for_test(test_config("https://gateway.local"));
    let result = dispatcher
        .execute(ActionRequest {
            action: "official_connector_get".into(),
            params: json!({"id": "console-1", "path": "/proxy/network/api/s/default/stat/sta"}),
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

#[tokio::test]
async fn official_list_clients_sends_expected_get_request() {
    let server = CaptureServer::spawn(200, r#"{"items":[]}"#);
    let dispatcher = ActionDispatcher::new_for_test(test_config(server.url()));

    dispatcher
        .execute(ActionRequest {
            action: "official_list_clients".into(),
            params: json!({"siteId": "site-1", "query": {"limit": 1}}),
        })
        .await
        .expect("official list clients should succeed");

    let request = server.request();
    assert!(request
        .starts_with("get /proxy/network/integration/v1/sites/site-1/clients?limit=1 http/1.1"));
    assert!(request.contains("x-api-key: test-key"));
}

#[tokio::test]
async fn official_create_network_sends_body() {
    let server = CaptureServer::spawn(201, r#"{"id":"network-1"}"#);
    let dispatcher = ActionDispatcher::new_for_test(test_config(server.url()));

    dispatcher
        .execute(ActionRequest {
            action: "official_create_network".into(),
            params: json!({"siteId": "site-1", "body": {"name": "IoT"}}),
        })
        .await
        .expect("official create network should succeed");

    let request = server.request();
    assert!(request.starts_with("post /proxy/network/integration/v1/sites/site-1/networks "));
    assert!(request.contains(r#""name":"iot""#));
}

#[tokio::test]
async fn official_path_params_accept_numbers_and_encode_segments() {
    let server = CaptureServer::spawn(200, r#"{"ok":true}"#);
    let dispatcher = ActionDispatcher::new_for_test(test_config(server.url()));

    dispatcher
        .execute(ActionRequest {
            action: "official_get_network_details".into(),
            params: json!({"siteId": "site-1", "networkId": "net/a?b"}),
        })
        .await
        .expect("encoded network details should succeed");

    let request = server.request();
    assert!(
        request.starts_with("get /proxy/network/integration/v1/sites/site-1/networks/net%2fa%3fb ")
    );

    let server = CaptureServer::spawn(200, r#"{"ok":true}"#);
    let dispatcher = ActionDispatcher::new_for_test(test_config(server.url()));
    dispatcher
        .execute(ActionRequest {
            action: "official_execute_port_action".into(),
            params: json!({
                "siteId": "site-1",
                "deviceId": "device-1",
                "portIdx": 1,
                "body": {"action": "cycle-poe"}
            }),
        })
        .await
        .expect("numeric port path parameter should succeed");

    let request = server.request();
    assert!(request.starts_with(
        "post /proxy/network/integration/v1/sites/site-1/devices/device-1/interfaces/ports/1/actions "
    ));
}

#[tokio::test]
async fn official_connector_get_allows_integration_proxy_path() {
    let server = CaptureServer::spawn(200, r#"{"ok":true}"#);
    let dispatcher = ActionDispatcher::new_for_test(test_config(server.url()));

    dispatcher
        .execute(ActionRequest {
            action: "official_connector_get".into(),
            params: json!({
                "id": "console-1",
                "path": "/proxy/network/integration/v1/sites"
            }),
        })
        .await
        .expect("connector get should allow integration proxy path");

    let request = server.request();
    assert!(request.starts_with(
        "get /proxy/network/integration/v1/connector/consoles/console-1/proxy/network/integration/v1/sites "
    ));
}

#[tokio::test]
async fn http_query_must_be_object() {
    let dispatcher = ActionDispatcher::new_for_test(test_config("https://gateway.local"));
    let result = dispatcher
        .execute(ActionRequest {
            action: "official_list_clients".into(),
            params: json!({"siteId": "site-1", "query": "limit=1"}),
        })
        .await;
    let message = result.unwrap_err().to_string();
    assert!(message.contains("query must be a JSON object"));
}

#[tokio::test]
async fn hybrid_defaults_to_internal_without_site_id() {
    let dispatcher = ActionDispatcher::new_for_test(test_config("https://gateway.local"));
    let result = dispatcher
        .execute(ActionRequest {
            action: "list_clients".into(),
            params: json!({}),
        })
        .await;
    let message = result.unwrap_err().to_string();
    assert!(message.contains("/proxy/network/api/s/default/stat/sta"));
}

#[tokio::test]
async fn hybrid_list_networks_defaults_to_registered_internal_action() {
    let server = CaptureServer::spawn(200, r#"{"data":[]}"#);
    let dispatcher = ActionDispatcher::new_for_test(test_config(server.url()));

    dispatcher
        .execute(ActionRequest {
            action: "list_networks".into(),
            params: json!({"prefer": "internal"}),
        })
        .await
        .expect("list_networks should resolve to an existing internal action");

    let request = server.request();
    assert!(request.starts_with("get /proxy/network/api/s/default/rest/networkconf "));
}

#[tokio::test]
async fn events_action_calls_rest_event_and_applies_limit() {
    let server = CaptureServer::spawn(200, r#"{"data":[{"id":1},{"id":2}]}"#);
    let dispatcher = ActionDispatcher::new_for_test(test_config(server.url()));

    let result = dispatcher
        .execute(ActionRequest {
            action: "events".into(),
            params: json!({"limit": 1}),
        })
        .await
        .expect("events should succeed");

    let request = server.request();
    assert!(request.starts_with("get /proxy/network/api/s/default/rest/event "));
    assert_eq!(result["data"].as_array().expect("data").len(), 1);
}

#[tokio::test]
async fn generated_internal_v2_action_uses_v2_site_prefix() {
    let server = CaptureServer::spawn(200, r#"{"data":[]}"#);
    let dispatcher = ActionDispatcher::new_for_test(test_config(server.url()));

    dispatcher
        .execute(ActionRequest {
            action: "unifi_list_acl_rules".into(),
            params: json!({}),
        })
        .await
        .expect("v2 internal action should succeed");

    let request = server.request();
    assert!(request.starts_with("get /proxy/network/v2/api/site/default/acl-rules "));
}

#[tokio::test]
async fn hybrid_uses_official_when_site_id_is_present() {
    let dispatcher = ActionDispatcher::new_for_test(test_config("https://gateway.local"));
    let result = dispatcher
        .execute(ActionRequest {
            action: "list_clients".into(),
            params: json!({"siteId": "site-1"}),
        })
        .await;
    let message = result.unwrap_err().to_string();
    assert!(message.contains("/proxy/network/integration/v1/sites/site-1/clients"));
}

#[test]
fn all_hybrid_aliases_resolve_to_expected_targets() {
    let cases = [
        ("list_clients", "clients", "official_list_clients"),
        ("list_devices", "devices", "official_list_devices"),
        (
            "list_networks",
            "unifi_list_networks",
            "official_list_networks",
        ),
        ("list_wifi", "wlans", "official_list_wifi"),
        ("get_system_info", "sysinfo", "official_get_info"),
    ];

    for (action, internal, official) in cases {
        let (target, params) = rustifi::actions::hybrid::resolve(action, &json!({})).unwrap();
        assert_eq!(target, internal, "{action} should default to internal");
        assert_eq!(params, json!({}));

        let (target, params) =
            rustifi::actions::hybrid::resolve(action, &json!({"siteId": "site-1"})).unwrap();
        assert_eq!(target, official, "{action} should use official with siteId");
        assert_eq!(params, json!({"siteId": "site-1"}));

        let (target, params) = rustifi::actions::hybrid::resolve(
            action,
            &json!({"siteId": "site-1", "prefer": "internal"}),
        )
        .unwrap();
        assert_eq!(target, internal, "{action} prefer=internal should win");
        assert_eq!(params, json!({"siteId": "site-1"}));
    }
}

#[test]
fn hybrid_preference_validation_is_explicit() {
    let (target, params) =
        rustifi::actions::hybrid::resolve("list_clients", &json!({"prefer": "official"})).unwrap();
    assert_eq!(target, "official_list_clients");
    assert_eq!(params, json!({}));

    let message = rustifi::actions::hybrid::resolve("list_clients", &json!({"prefer": "maybe"}))
        .unwrap_err()
        .to_string();
    assert!(message.contains("unknown hybrid preference"));
}

struct CaptureServer {
    addr: std::net::SocketAddr,
    handle: std::thread::JoinHandle<String>,
}

impl CaptureServer {
    fn spawn(status: u16, body: &'static str) -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind capture server");
        let addr = listener.local_addr().expect("capture server addr");
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept one request");
            let mut request = Vec::new();
            let mut buffer = [0_u8; 1024];
            loop {
                let read = std::io::Read::read(&mut stream, &mut buffer).expect("read request");
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..read]);
                if body_complete(&request) {
                    break;
                }
            }
            let response = format!(
                "HTTP/1.1 {status} OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{body}",
                body.len()
            );
            std::io::Write::write_all(&mut stream, response.as_bytes()).expect("write response");
            String::from_utf8_lossy(&request).to_ascii_lowercase()
        });
        Self { addr, handle }
    }

    fn url(&self) -> String {
        format!("http://{}", self.addr)
    }

    fn request(self) -> String {
        self.handle.join().expect("capture thread should finish")
    }
}

fn body_complete(request: &[u8]) -> bool {
    let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") else {
        return false;
    };
    let headers = String::from_utf8_lossy(&request[..header_end]).to_ascii_lowercase();
    let content_length = headers
        .lines()
        .find_map(|line| line.strip_prefix("content-length: "))
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(0);
    request.len() >= header_end + 4 + content_length
}

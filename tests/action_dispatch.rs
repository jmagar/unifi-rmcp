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
async fn mutating_official_action_requires_confirmation() {
    let dispatcher = ActionDispatcher::new_for_test(test_config("https://gateway.local"));
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
    let dispatcher = ActionDispatcher::new_for_test(test_config("https://gateway.local"));
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

#[tokio::test]
async fn official_list_clients_sends_expected_get_request() {
    let server = CaptureServer::spawn(200, r#"{"items":[]}"#);
    let dispatcher = ActionDispatcher::new_for_test(test_config(server.url()));

    dispatcher
        .execute(ActionRequest {
            action: "official_list_clients".into(),
            params: json!({"siteId": "site-1", "query": {"limit": 1}}),
            confirm: false,
        })
        .await
        .expect("official list clients should succeed");

    let request = server.request();
    assert!(request
        .starts_with("get /proxy/network/integration/v1/sites/site-1/clients?limit=1 http/1.1"));
    assert!(request.contains("x-api-key: test-key"));
}

#[tokio::test]
async fn official_create_network_requires_confirm_and_sends_body() {
    let server = CaptureServer::spawn(201, r#"{"id":"network-1"}"#);
    let dispatcher = ActionDispatcher::new_for_test(test_config(server.url()));

    dispatcher
        .execute(ActionRequest {
            action: "official_create_network".into(),
            params: json!({"siteId": "site-1", "body": {"name": "IoT"}}),
            confirm: true,
        })
        .await
        .expect("official create network should succeed with confirm");

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
            confirm: false,
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
            confirm: true,
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
            confirm: false,
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
            confirm: false,
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
            confirm: false,
        })
        .await;
    let message = result.unwrap_err().to_string();
    assert!(message.contains("/proxy/network/api/s/default/stat/sta"));
}

#[tokio::test]
async fn hybrid_uses_official_when_site_id_is_present() {
    let dispatcher = ActionDispatcher::new_for_test(test_config("https://gateway.local"));
    let result = dispatcher
        .execute(ActionRequest {
            action: "list_clients".into(),
            params: json!({"siteId": "site-1"}),
            confirm: false,
        })
        .await;
    let message = result.unwrap_err().to_string();
    assert!(message.contains("/proxy/network/integration/v1/sites/site-1/clients"));
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

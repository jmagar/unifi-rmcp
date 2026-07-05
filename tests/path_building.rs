use rustifi::api::internal::InternalNetworkApi;
use rustifi::api::official::OfficialNetworkApi;

#[test]
fn official_network_paths_use_integration_prefix() {
    let api = OfficialNetworkApi::new_for_test("https://gateway.local");
    assert_eq!(api.path("/v1/sites"), "/proxy/network/integration/v1/sites");
    assert_eq!(
        api.path("v1/sites/site-1/clients"),
        "/proxy/network/integration/v1/sites/site-1/clients"
    );
}

#[test]
fn internal_v1_paths_use_site_prefix() {
    let api = InternalNetworkApi::new_for_test("https://gateway.local", "default", false);
    assert_eq!(
        api.v1_site_path("stat/sta"),
        "/proxy/network/api/s/default/stat/sta"
    );
}

#[test]
fn internal_v2_paths_use_site_prefix() {
    let api = InternalNetworkApi::new_for_test("https://gateway.local", "default", false);
    assert_eq!(
        api.v2_site_path("firewall-policies"),
        "/proxy/network/v2/api/site/default/firewall-policies"
    );
}

#[test]
fn legacy_internal_v1_paths_skip_proxy_prefix() {
    let api = InternalNetworkApi::new_for_test("https://legacy.local:8443", "default", true);
    assert_eq!(api.v1_site_path("stat/sta"), "/api/s/default/stat/sta");
}

#[test]
fn shared_path_substitution_encodes_segments() {
    let params = serde_json::json!({"siteId": "site one", "clientId": "aa:bb:cc"});
    let path =
        rustifi::api::path::substitute_path("/v1/sites/{siteId}/clients/{clientId}", &params, &[])
            .unwrap();
    assert_eq!(path, "/v1/sites/site%20one/clients/aa%3Abb%3Acc");
}

#[test]
fn connector_path_rejects_bypass_shapes() {
    for candidate in [
        "/api/self",
        "https://example.test/proxy/network/integration/v1/info",
        "//example.test/proxy/network/integration/v1/info",
        "/proxy/network/integration/../api/self",
        "/proxy/network/integration/%2e%2e/api/self",
        "/proxy/network/integration/%2fapi/self",
        "/proxy/network/integration/%5capi/self",
        "/proxy/network/integration/v1/info?x=1",
        "/proxy/network/integration/v1/info#x",
    ] {
        let err = rustifi::api::path::validate_connector_path(
            candidate,
            &["/proxy/network/integration/", "/proxy/protect/integration/"],
        )
        .unwrap_err()
        .to_string();
        assert!(
            err.contains("outside the supported integration API prefix")
                || err.contains("unsafe connector path")
        );
    }
}

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

use std::process::Command;

#[test]
fn safe_live_fails_when_request_budget_is_exhausted() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["verify-api-endpoints", "--mode", "safe_live"])
        .current_dir("..")
        .env("UNIFI_URL", "https://unifi.example.invalid")
        .env("UNIFI_API_KEY", "test-key")
        .env("UNIFI_SITE", "default")
        .env("UNIFI_SITE_ID", "00000000-0000-4000-8000-000000000000")
        .env("UNIFI_VERIFY_MAX_REQUESTS", "0")
        .output()
        .expect("xtask safe live verifier should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("exhausted request budget"),
        "stderr did not contain budget failure: {stderr}"
    );
}

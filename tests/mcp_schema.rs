use rustifi::mcp::schemas::tool_definitions;

#[test]
fn schema_contains_official_and_internal_actions() {
    let tools = tool_definitions();
    let schema = &tools[0]["inputSchema"]["properties"]["action"]["enum"];
    let actions = schema.as_array().expect("action enum");

    assert!(actions.iter().any(|value| value == "clients"));
    assert!(actions.iter().any(|value| value == "official_list_clients"));
    assert!(actions.iter().any(|value| value == "internal_list_alarms"));
}

#[test]
fn schema_exposes_confirmation_parameter() {
    let tools = tool_definitions();
    assert!(tools[0]["inputSchema"]["properties"]
        .get("confirm")
        .is_some());
}

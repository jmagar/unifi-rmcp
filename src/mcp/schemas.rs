use serde_json::{json, Value};

use crate::capabilities::all_capabilities;

pub fn tool_definitions() -> Vec<Value> {
    let mut actions = all_capabilities()
        .iter()
        .map(|cap| cap.action.clone())
        .collect::<Vec<_>>();
    actions.push("help".to_string());
    actions.sort();
    actions.dedup();

    vec![json!({
        "name": "unifi",
        "description": "Query and manage a UniFi network controller via official, internal, and hybrid API actions. Mutating actions require confirm=true.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "Operation to perform.",
                    "enum": actions
                },
                "params": {
                    "type": "object",
                    "description": "Action-specific parameters, including path values, query, and body."
                },
                "confirm": {
                    "type": "boolean",
                    "description": "Required true for mutating actions."
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results to return (events only).",
                    "minimum": 1
                }
            },
            "required": ["action"]
        }
    })]
}

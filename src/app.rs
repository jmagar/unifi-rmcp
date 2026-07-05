use anyhow::Result;
use serde_json::Value;

use crate::actions::{ActionDispatcher, ActionRequest};
use crate::unifi::UnifiClient;

/// Business service layer. All logic lives here.
/// CLI and MCP are thin shims that call into this.
#[derive(Clone)]
pub struct UnifiService {
    client: UnifiClient,
}

impl UnifiService {
    pub fn new(client: UnifiClient) -> Self {
        Self { client }
    }

    pub async fn clients(&self) -> Result<Value> {
        self.client.clients().await
    }

    pub async fn devices(&self) -> Result<Value> {
        self.client.devices().await
    }

    pub async fn wlans(&self) -> Result<Value> {
        self.client.wlans().await
    }

    pub async fn health(&self) -> Result<Value> {
        self.client.health().await
    }

    pub async fn alarms(&self) -> Result<Value> {
        self.client.alarms().await
    }

    pub async fn events(&self, limit: Option<usize>) -> Result<Value> {
        let mut events = self.client.events().await?;
        truncate_data_array(&mut events, limit);
        Ok(events)
    }

    pub async fn sysinfo(&self) -> Result<Value> {
        self.client.sysinfo().await
    }

    pub async fn me(&self) -> Result<Value> {
        self.client.me().await
    }

    pub async fn execute(&self, request: ActionRequest) -> Result<Value> {
        ActionDispatcher::new(self.client.config())
            .execute(request)
            .await
    }
}

fn truncate_data_array(value: &mut Value, limit: Option<usize>) {
    let Some(limit) = limit else {
        return;
    };
    if let Some(items) = value.get_mut("data").and_then(Value::as_array_mut) {
        items.truncate(limit);
    }
}

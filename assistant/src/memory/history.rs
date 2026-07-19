use crate::Exchange;
use serde::{Deserialize, Serialize};
use std::path::Path;

const MAX_HISTORY: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConversationHistory {
    exchanges: Vec<Exchange>,
}

impl ConversationHistory {
    pub fn push(&mut self, exchange: Exchange) {
        self.exchanges.push(exchange);
        // Keep only the last MAX_HISTORY exchanges
        if self.exchanges.len() > MAX_HISTORY {
            let drain = self.exchanges.len() - MAX_HISTORY;
            self.exchanges.drain(..drain);
        }
    }

    pub fn recent(&self, n: usize) -> Vec<Exchange> {
        let start = self.exchanges.len().saturating_sub(n);
        self.exchanges[start..].to_vec()
    }

    pub async fn load(data_dir: &Path) -> anyhow::Result<Self> {
        let path = data_dir.join("history.json");
        if path.exists() {
            let data = tokio::fs::read_to_string(&path).await?;
            Ok(serde_json::from_str(&data).unwrap_or_default())
        } else {
            Ok(Self::default())
        }
    }

    pub async fn save(&self, data_dir: &Path) -> anyhow::Result<()> {
        let path = data_dir.join("history.json");
        let data = serde_json::to_string_pretty(self)?;
        tokio::fs::write(&path, data).await?;
        Ok(())
    }
}

pub mod history;
pub mod preferences;

use crate::Exchange;
use history::ConversationHistory;
use preferences::Preferences;
use std::path::PathBuf;
use tokio::sync::RwLock;

pub struct Memory {
    history: RwLock<ConversationHistory>,
    preferences: RwLock<Preferences>,
    data_dir: PathBuf,
}

impl Memory {
    pub async fn new() -> anyhow::Result<Self> {
        let data_dir = data_dir();

        // Ensure data directory exists
        tokio::fs::create_dir_all(&data_dir).await?;

        let history = ConversationHistory::load(&data_dir).await?;
        let preferences = Preferences::load(&data_dir).await?;

        Ok(Self {
            history: RwLock::new(history),
            preferences: RwLock::new(preferences),
            data_dir,
        })
    }

    pub async fn record_exchange(&self, exchange: Exchange) -> anyhow::Result<()> {
        let mut history = self.history.write().await;
        history.push(exchange);
        history.save(&self.data_dir).await
    }

    /// Persist a durable user fact (agentic memory). Replaces any existing
    /// fact with the same key. Saved to disk immediately.
    pub async fn record_fact(&self, key: &str, value: &str) -> anyhow::Result<()> {
        let mut prefs = self.preferences.write().await;
        prefs.facts.insert(key.to_string(), value.to_string());
        prefs.save(&self.data_dir).await
    }

    /// Retrieve a durable user fact by key.
    pub async fn fact(&self, key: &str) -> Option<String> {
        let prefs = self.preferences.read().await;
        prefs.facts.get(key).cloned()
    }

    /// Compact block of remembered facts for LLM context injection.
    pub async fn memory_block(&self) -> String {
        let prefs = self.preferences.read().await;
        prefs.memory_block()
    }

    pub async fn context(&self) -> ConversationContext {
        let history = self.history.read().await;
        let prefs = self.preferences.read().await;

        ConversationContext {
            recent_exchanges: history.recent(5),
            user_name: prefs.user_name.clone(),
            voice: prefs.voice.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConversationContext {
    pub recent_exchanges: Vec<Exchange>,
    pub user_name: Option<String>,
    pub voice: Option<String>,
}

impl ConversationContext {
    /// Build a short text summary of recent conversation for context injection.
    pub fn recent_summary(&self) -> String {
        if self.recent_exchanges.is_empty() {
            return String::new();
        }
        let mut lines = Vec::new();
        for ex in &self.recent_exchanges {
            lines.push(format!("User: {}", ex.user_input));
            // Truncate long responses to keep context compact
            let resp = if ex.response.len() > 120 {
                format!(
                    "{}…",
                    &ex.response[..ex
                        .response
                        .char_indices()
                        .nth(120)
                        .map_or(ex.response.len(), |(i, _)| i)]
                )
            } else {
                ex.response.clone()
            };
            lines.push(format!("Lai: {resp}"));
        }
        if let Some(name) = &self.user_name {
            lines.insert(0, format!("The user's name is {name}."));
        }
        lines.join("\n")
    }
}

fn data_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".lai").join("assistant")
    } else {
        PathBuf::from("/tmp").join(".lai-assistant")
    }
}

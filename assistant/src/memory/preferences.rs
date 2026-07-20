use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    pub user_name: Option<String>,
    pub voice: Option<String>,
    pub language: String,
    pub wake_word: String,
    pub proactive_enabled: bool,
    /// Durable user facts the agent has learned (e.g. "prefers Celsius",
    /// "works in Berlin"). Persisted across sessions — the agentic memory.
    #[serde(default)]
    pub facts: HashMap<String, String>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            user_name: None,
            voice: Some("en_US-lessac-medium".to_string()),
            language: "en".to_string(),
            wake_word: "hey lai".to_string(),
            proactive_enabled: true,
            facts: HashMap::new(),
        }
    }
}

impl Preferences {
    /// Build a compact natural-language block of remembered facts for context
    /// injection into the LLM system prompt.
    pub fn memory_block(&self) -> String {
        let mut lines = Vec::new();
        if let Some(name) = &self.user_name {
            lines.push(format!("The user's name is {name}."));
        }
        // Determinism rule: HashMap iteration order is unstable, so sort by key
        // before emitting. Otherwise the injected context (and any snapshot of
        // it) would reorder run-to-run.
        let mut facts: Vec<(&String, &String)> = self.facts.iter().collect();
        facts.sort_by(|a, b| a.0.cmp(b.0));
        for (k, v) in facts {
            lines.push(format!("- {k}: {v}"));
        }
        if lines.is_empty() {
            String::new()
        } else {
            format!("Known about the user:\n{}", lines.join("\n"))
        }
    }
}

impl Preferences {
    pub async fn load(data_dir: &Path) -> anyhow::Result<Self> {
        let path = data_dir.join("preferences.toml");
        if path.exists() {
            let data = tokio::fs::read_to_string(&path).await?;
            Ok(toml::from_str(&data).unwrap_or_default())
        } else {
            let prefs = Self::default();
            prefs.save(data_dir).await?;
            Ok(prefs)
        }
    }

    pub async fn save(&self, data_dir: &Path) -> anyhow::Result<()> {
        let path = data_dir.join("preferences.toml");
        let data = toml::to_string_pretty(self)?;
        tokio::fs::write(&path, data).await?;
        Ok(())
    }
}

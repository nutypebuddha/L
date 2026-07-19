pub mod actions;
pub mod agent;
pub mod apps;
pub mod engines;
pub mod http;
#[cfg(feature = "mcp")]
pub mod mcp_server;
pub mod intent;
pub mod llm;
pub mod mcp_client;
pub mod memory;
pub mod multimodal;
pub mod personality;
pub mod proactive;
pub mod speech;

use anyhow::Result;
use std::sync::Arc;

use crate::engines::Engines;
use crate::intent::schema::Intent;
use crate::memory::Memory;

/// Top-level assistant handle.
pub struct Assistant {
    memory: Memory,
    action_registry: actions::Registry,
    engines: Engines,
    wake_word: String,
    pid_file: std::path::PathBuf,
}

/// A single exchange: user input → assistant response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Exchange {
    pub user_input: String,
    pub intent: Intent,
    pub response: String,
}

/// Structured JSON response from the assistant (for `--format json`).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssistantResponse {
    pub response: String,
    pub source: String,
    pub intent: Intent,
    pub toast: String,
}

impl AssistantResponse {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            serde_json::json!({
                "response": self.response,
                "source": self.source,
                "toast": self.toast,
            })
            .to_string()
        })
    }
}

impl Assistant {
    pub async fn new() -> Result<Self> {
        Self::with_engines(Engines::noop()).await
    }

    pub async fn with_engines(engines: Engines) -> Result<Self> {
        Self::with_config(engines, "lai").await
    }

    pub async fn with_config(engines: Engines, wake_word: &str) -> Result<Self> {
        let memory = Memory::new().await?;
        let action_registry = actions::Registry::new();
        let pid_file = dirs_pid_file();

        Ok(Self {
            memory,
            action_registry,
            engines,
            wake_word: wake_word.to_string(),
            pid_file,
        })
    }

    /// Run the full voice loop: listen → transcribe → route → speak.
    pub async fn run_voice_loop(&self) -> Result<()> {
        self.run_voice_loop_inner(false).await
    }

    /// Run daemon mode: wake word detection → process → loop.
    /// Writes PID file, provides toast feedback, runs until killed.
    pub async fn run_daemon(&self) -> Result<()> {
        // Write PID file
        if let Some(parent) = self.pid_file.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }
        tokio::fs::write(&self.pid_file, std::process::id().to_string()).await?;

        eprintln!("[assistant] daemon started (pid {})", std::process::id());
        eprintln!("[assistant] wake word: \"{}\"", self.wake_word);
        eprintln!(
            "[assistant] say \"{} help\" for commands, kill {} to stop",
            self.wake_word,
            std::process::id()
        );

        // Ensure wake lock
        let _ = tokio::process::Command::new("termux-wake-lock")
            .output()
            .await;

        // Greet on start
        toast(personality::GREETING).await;
        speech::tts::speak(personality::GREETING).await?;

        self.run_voice_loop_inner(true).await
    }

    async fn run_voice_loop_inner(&self, daemon: bool) -> Result<()> {
        let wake_lower = self.wake_word.to_lowercase();

        loop {
            eprintln!("[listening...]");
            let wav_path = speech::capture::capture_until_silence(5, 300).await?;
            let text = speech::stt::transcribe(&wav_path).await?;
            let _ = tokio::fs::remove_file(&wav_path).await;

            if text.trim().is_empty() {
                continue;
            }

            let text_lower = text.to_lowercase();

            // In daemon mode, only respond to wake word
            if daemon {
                if let Some(cmd) = text_lower.strip_prefix(&wake_lower) {
                    let cmd = cmd
                        .trim()
                        .trim_start_matches(|c: char| c == ',' || c == ':')
                        .trim();
                    if cmd.is_empty() {
                        toast(personality::WAKE_YES).await;
                        speech::tts::speak(personality::WAKE_YES).await?;
                        continue;
                    }
                    self.process_command(cmd, &text).await?;
                } else if text_lower == wake_lower {
                    toast(personality::WAKE_YES).await;
                    speech::tts::speak(personality::WAKE_YES).await?;
                }
                // else: ignore non-wake-word input
                continue;
            }

            // Non-daemon: process everything
            self.process_command(&text, &text).await?;
        }
    }

    async fn process_command(&self, input: &str, raw: &str) -> Result<()> {
        eprintln!("[you: {input}]");

        let intent = intent::classify(input);
        let response = self.route_intent(&intent, raw).await;

        eprintln!("[lai: {response}]");

        // Toast for short responses
        toast(&personality::toast_line(&response)).await;

        speech::tts::speak(&response).await?;

        self.memory
            .record_exchange(Exchange {
                user_input: input.to_string(),
                intent,
                response,
            })
            .await?;

        Ok(())
    }

    /// Process a single text input and return the response.
    pub async fn process_text(&self, input: &str) -> Result<String> {
        let intent = intent::classify(input);
        let ctx = self.memory.context().await;
        let context_str = ctx.recent_summary();

        let response = if !context_str.is_empty() {
            self.route_intent_with_context(&intent, input, &context_str)
                .await
        } else {
            self.route_intent(&intent, input).await
        };

        self.memory
            .record_exchange(Exchange {
                user_input: input.to_string(),
                intent: intent.clone(),
                response: response.clone(),
            })
            .await?;

        Ok(response)
    }

    /// Process text and return a structured JSON response.
    pub async fn process_text_json(&self, input: &str) -> Result<AssistantResponse> {
        let intent = intent::classify(input);
        let ctx = self.memory.context().await;
        let context_str = ctx.recent_summary();

        let response = if !context_str.is_empty() {
            self.route_intent_with_context(&intent, input, &context_str)
                .await
        } else {
            self.route_intent(&intent, input).await
        };

        let toast = personality::toast_line(&response);

        self.memory
            .record_exchange(Exchange {
                user_input: input.to_string(),
                intent: intent.clone(),
                response: response.clone(),
            })
            .await?;

        Ok(AssistantResponse {
            response,
            source: "lai-assistant".into(),
            intent,
            toast,
        })
    }

    /// Start an HTTP server on localhost that proxies chat requests to this
    /// assistant. Used by the Android app to communicate without exec/SELinux
    /// restrictions.
    pub async fn serve_http(self: Arc<Self>, port: u16) -> Result<()> {
        http::serve(self, port).await
    }

    async fn route_intent(&self, intent: &Intent, _raw_input: &str) -> String {
        self.route_intent_with_context(intent, _raw_input, "").await
    }

    async fn route_intent_with_context(
        &self,
        intent: &Intent,
        _raw_input: &str,
        _context: &str,
    ) -> String {
        let ret = match intent {
            // ── System actions ─────────────────────────────────
            Intent::SetTimer {
                duration_secs,
                label,
            } => actions::timer::set_timer(*duration_secs, label.as_deref()).await,
            Intent::CancelTimer { label } => actions::timer::cancel_timer(label.as_deref()).await,
            Intent::SetAlarm { time } => actions::alarm::set_alarm(time).await,
            Intent::SetReminder { text, when } => actions::reminder::set_reminder(text, when).await,
            #[cfg(feature = "termux")]
            Intent::SendMessage { contact, message } => {
                actions::sms::send_message(contact, message).await
            }
            #[cfg(feature = "termux")]
            Intent::Call { contact } => actions::call::call_contact(contact).await,
            #[cfg(feature = "termux")]
            Intent::TakePhoto => actions::camera::take_photo().await,
            #[cfg(feature = "termux")]
            Intent::BatteryStatus => actions::battery::status().await,
            #[cfg(feature = "termux")]
            Intent::GetLocation => actions::location::get_location().await,
            #[cfg(feature = "termux")]
            Intent::SetClipboard { text } => actions::notification::set_clipboard(text).await,
            #[cfg(feature = "termux")]
            Intent::GetClipboard => actions::notification::get_clipboard().await,

            // ── Ecosystem: Proof (reasoning) ───────────────────
            Intent::Solve { query } => {
                let result = (self.engines.solve)(query.clone());
                format!("Proof reasoning:\n{result}")
            }

            // ── Ecosystem: Gate (validation) ──────────────────
            Intent::Validate { text, context } => {
                let result = (self.engines.validate)((text.clone(), context.clone()));
                format!("Gate validation:\n{result}")
            }
            Intent::Score { text } => {
                let result = (self.engines.score)(text.clone());
                format!("Confidence score:\n{result}")
            }
            Intent::Fix { text, context } => {
                let result = (self.engines.fix)((text.clone(), context.clone()));
                format!("Fixed response:\n{result}")
            }

            // ── Ecosystem: Tanto (compute) ────────────────────
            Intent::Eval { expression } => {
                let result = (self.engines.eval)(expression.clone());
                format!("Result: {result}")
            }
            Intent::Convert { value, from, to } => {
                let result = (self.engines.convert)((*value, from.clone(), to.clone()));
                format!("Conversion: {result}")
            }
            Intent::Formula { name, args } => {
                let result = (self.engines.formula)((name.clone(), args.clone()));
                format!("Formula result:\n{result}")
            }

            // ── Ecosystem: Athena (relational) ────────────────
            Intent::SearchFormulas { keyword } => {
                let result = (self.engines.search_formulas)(keyword.clone());
                format!("Formula search:\n{result}")
            }
            Intent::Traverse { domain, depth } => {
                let result = (self.engines.traverse)((domain.clone(), *depth));
                format!("Wheel traversal:\n{result}")
            }
            Intent::Classify { token } => {
                let result = (self.engines.classify)(token.clone());
                format!("Classification:\n{result}")
            }
            Intent::Wheel { domain } => {
                let result = (self.engines.wheel)(domain.clone());
                format!("Zodiac wheel:\n{result}")
            }
            Intent::Reason {
                have,
                want,
                max_depth,
            } => {
                let result = (self.engines.reason)((have.clone(), want.clone(), *max_depth));
                format!("Reasoning path:\n{result}")
            }
            Intent::Shikai { query } => {
                let result = (self.engines.shikai)(query.clone());
                format!("Shikai NLP:\n{result}")
            }
            Intent::BankaiSolve { query } => {
                let result = (self.engines.bankai_solve)(query.clone());
                format!("Bankai solve:\n{result}")
            }
            Intent::EvalFormula { formula_id, args } => {
                let result = (self.engines.eval_formula)((formula_id.clone(), args.clone()));
                format!("Formula evaluation:\n{result}")
            }
            Intent::ChainFormulas { formulas, args } => {
                let result = (self.engines.chain_formulas)((formulas.clone(), args.clone()));
                format!("Formula chain:\n{result}")
            }

            // ── Generic fallbacks ─────────────────────────────
            Intent::Query { text } => {
                let lowered = text.to_lowercase();
                if lowered.contains("who are you")
                    || lowered.contains("what are you")
                    || lowered.contains("how are you")
                    || lowered.contains("your name")
                {
                    return "I am L — the Shadow Monarch. I do not guess; I observe, reason, and act. Name the task: I will solve it, verify it, or tell you plainly what I cannot.".to_string();
                }
                // Agent loop first: the model may answer directly or chain
                // tools (solve/validate/reminder/etc.) to complete the goal.
                let mem = self.memory.memory_block().await;
                if let Some(answer) = agent::run_opt(text, &mem).await {
                    return answer;
                }
                // Fall back to the local LLM for a natural answer; then to the
                // deterministic reasoning engine if the model is unavailable.
                if let Some(answer) = llm::Llm.answer(text, "") {
                    return answer;
                }
                let result = (self.engines.solve)(text.clone());
                format!("{result}")
            }
            Intent::Conversational { text } => {
                let lowered = text.to_lowercase();
                let lower = lowered.trim();
                // Greetings get a real greeting, not an echo
                if matches!(
                    lower,
                    "hello" | "hi" | "hey" | "hey lai" | "greetings" | "yo" | "sup"
                ) {
                    return personality::GREETING.to_string();
                }
                if lower.contains("how are you") || lower.contains("who are you") {
                    return "I am L — the Shadow Monarch. I do not guess; I observe, reason, and act. Name the task: I will solve it, verify it, or tell you plainly what I cannot.".to_string();
                }
                // Agent loop first: natural reply, or chain tools as needed.
                let mem = self.memory.memory_block().await;
                if let Some(answer) = agent::run_opt(text, &mem).await {
                    return answer;
                }
                // Try the local LLM for a natural, conversational reply; fall
                // back to the reasoning engine if no model is present.
                if let Some(answer) = llm::Llm.answer(text, "") {
                    return answer;
                }
                // Otherwise, actually reason about it via the Proof engine
                let result = (self.engines.solve)(text.clone());
                let trimmed = result.trim();
                if trimmed.is_empty() {
                    format!("I don't have a verified answer for that yet. Try: \"solve 2+2\", \"validate water is H2O\", \"search formulas for gravity\", or \"what's my battery\".")
                } else {
                    format!("{trimmed}")
                }
            }
            Intent::Goodbye => {
                let goodbye_msg = personality::GOODBYE.to_string();
                speech::tts::speak(&goodbye_msg).await.ok();
                goodbye_msg
            }
            Intent::Help => self.help_text(),
            Intent::Unknown => personality::CONFUSED.to_string(),
        };
        personality::humanize(&ret)
    }

    fn help_text(&self) -> String {
        let mut text = String::from(
            "Here's what I can do:\n\
             \n\
             System:\n\
             \x20 • Set timers: \"set a 5 minute timer\"\n\
             \x20 • Set alarms: \"set an alarm for 7am\"\n\
             \x20 • Set reminders: \"remind me to call mom at 3pm\"\n\
             \x20 • Send texts: \"text John hello\"\n\
             \x20 • Make calls: \"call Sarah\"\n\
             \x20 • Take photos: \"take a photo\"\n\
             \x20 • Check battery: \"what's my battery level\"\n\
             \x20 • Get location: \"where am I\"\n\
             \x20 • Clipboard: \"copy this to clipboard\" or \"read clipboard\"\n\
             \n\
             Proof (reasoning):\n\
             \x20 • Solve: \"prove that 2+2=4\"\n\
             \x20 • Reason: \"reason about climate change\"\n\
             \n\
             Gate (validation):\n\
             \x20 • Validate: \"validate this claim against that\"\n\
             \x20 • Score: \"score this response\"\n\
             \x20 • Fix: \"fix this response against the context\"\n\
             \n\
             Tanto (compute):\n\
             \x20 • Calculate: \"calculate 2+2\" / \"what's sqrt(144)\"\n\
             \x20 • Convert: \"convert 100 miles to kilometers\"\n\
             \x20 • Formula: \"evaluate the circle_area formula with r=5\"\n\
             \n\
             Athena (relational):\n\
             \x20 • Search: \"search formulas for gravity\"\n\
             \x20 • Traverse: \"traverse the mangala domain\"\n\
             \x20 • Classify: \"classify mercury\"\n\
             \x20 • Wheel: \"show the wheel\" / \"wheel aries\"\n\
             \x20 • Reason: \"reason from acceleration to distance\"\n\
             \x20 • Shikai: \"shikai what is gravity\"\n\
             \x20 • Bankai: \"bankai solve for the meaning of life\"\n\
             \x20 • Chain: \"chain formulas circle_area, sphere_volume\"\n\
             \n\
             General:\n\
             \x20 • Ask: \"what is the capital of France\"\n\
             \x20 • Goodbye: \"goodbye\"",
        );
        text.push_str("\n\nRegistered actions:\n");
        for (name, desc) in self.action_registry.list() {
            text.push_str(&format!("\x20 • {name}: {desc}\n"));
        }
        text
    }
}

// ── Helpers ────────────────────────────────────────────────────

/// PID file path for daemon mode: ~/.lai/assistant.pid
fn dirs_pid_file() -> std::path::PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        std::path::PathBuf::from(home)
            .join(".lai")
            .join("assistant.pid")
    } else {
        std::path::PathBuf::from("/tmp/lai-assistant.pid")
    }
}

/// Show a toast notification via termux-toast (best-effort, non-blocking).
async fn toast(msg: &str) {
    let _ = tokio::process::Command::new("termux-toast")
        .arg(msg)
        .output()
        .await;
}

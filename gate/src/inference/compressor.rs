/// Prompt compressor - reduces token count without losing meaning.
/// Zero dependencies, pattern-based compression.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    Light,
    Medium,
    Aggressive,
}

pub struct PromptCompressor {
    level: CompressionLevel,
}

impl PromptCompressor {
    pub fn new(level: CompressionLevel) -> Self {
        PromptCompressor { level }
    }

    pub fn light() -> Self {
        Self::new(CompressionLevel::Light)
    }

    pub fn medium() -> Self {
        Self::new(CompressionLevel::Medium)
    }

    pub fn aggressive() -> Self {
        Self::new(CompressionLevel::Aggressive)
    }

    /// Compress a prompt, returning compressed version and token savings.
    pub fn compress(&self, prompt: &str) -> (String, CompressionStats) {
        let original_tokens = estimate_tokens(prompt);
        let mut result = prompt.to_string();

        // Phase 1: Normalize whitespace (always)
        result = normalize_whitespace(&result);

        // Phase 2: Remove filler words (medium + aggressive)
        if matches!(
            self.level,
            CompressionLevel::Medium | CompressionLevel::Aggressive
        ) {
            result = remove_filler_words(&result);
        }

        // Phase 3: Compress polite phrases (medium + aggressive)
        if matches!(
            self.level,
            CompressionLevel::Medium | CompressionLevel::Aggressive
        ) {
            result = compress_polite_phrases(&result);
        }

        // Phase 4: Simplify redundancy (aggressive)
        if self.level == CompressionLevel::Aggressive {
            result = simplify_redundancy(&result);
        }

        // Phase 5: Compress instructions (aggressive)
        if self.level == CompressionLevel::Aggressive {
            result = compress_instructions(&result);
        }

        let compressed_tokens = estimate_tokens(&result);
        let stats = CompressionStats {
            original_tokens,
            compressed_tokens,
            saved_tokens: original_tokens.saturating_sub(compressed_tokens),
            saved_percent: if original_tokens > 0 {
                ((original_tokens.saturating_sub(compressed_tokens)) as f64
                    / original_tokens as f64)
                    * 100.0
            } else {
                0.0
            },
        };

        (result, stats)
    }

    /// Compress for a specific context (code, chat, analysis).
    pub fn compress_for(&self, prompt: &str, context: &str) -> (String, CompressionStats) {
        let mut result = prompt.to_string();

        // Context-specific compression
        let lower = context.to_lowercase();
        if lower.contains("code") || lower.contains("programming") {
            result = compress_code_context(&result);
        } else if lower.contains("creative") || lower.contains("writing") {
            result = compress_creative_context(&result);
        } else if lower.contains("analysis") || lower.contains("research") {
            result = compress_analysis_context(&result);
        }

        // General compression
        self.compress(&result)
    }
}

impl Default for PromptCompressor {
    fn default() -> Self {
        Self::medium()
    }
}

#[derive(Debug, Clone)]
pub struct CompressionStats {
    pub original_tokens: usize,
    pub compressed_tokens: usize,
    pub saved_tokens: usize,
    pub saved_percent: f64,
}

/// Estimate token count (rough: 1 token ≈ 4 chars or ≈ 0.75 words).
pub fn estimate_tokens(text: &str) -> usize {
    let chars = text.len();
    let words = text.split_whitespace().count();
    // Use whichever is more conservative
    let by_chars = chars.div_ceil(4);
    let by_words = (words * 4).div_ceil(4);
    by_chars.max(by_words)
}

fn normalize_whitespace(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut prev_was_space = false;

    for ch in text.chars() {
        if ch.is_whitespace() {
            if !prev_was_space {
                result.push(' ');
            }
            prev_was_space = true;
        } else {
            result.push(ch);
            prev_was_space = false;
        }
    }

    result.trim().to_string()
}

fn remove_filler_words(text: &str) -> String {
    static FILLER_WORDS: &[&str] = &[
        "please",
        "kindly",
        "just",
        "really",
        "very",
        "quite",
        "actually",
        "basically",
        "simply",
        "literally",
        "honestly",
        "frankly",
        "obviously",
        "clearly",
        "definitely",
        "certainly",
        "absolutely",
        "essentially",
        "fundamentally",
    ];

    let mut result = text.to_string();
    for filler in FILLER_WORDS {
        // Remove filler word with surrounding space
        let patterns = [
            format!(" {} ", filler),
            format!(", {} ", filler),
            format!(" {}.", filler),
            format!(" {},", filler),
        ];
        for pattern in &patterns {
            result = result.replace(pattern, " ");
        }
    }

    // Clean up double spaces
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }

    result.trim().to_string()
}

fn compress_polite_phrases(text: &str) -> String {
    static POLITE_PATTERNS: &[(&str, &str)] = &[
        ("I would like you to ", ""),
        ("I want you to ", ""),
        ("Could you please ", ""),
        ("Would you please ", ""),
        ("Can you please ", ""),
        ("I need you to ", ""),
        ("I was wondering if you could ", ""),
        ("I was hoping you could ", ""),
        ("It would be great if you could ", ""),
        ("It would be helpful if you could ", ""),
        ("I would appreciate it if you could ", ""),
        ("Please help me to ", ""),
        ("Please assist me in ", ""),
    ];

    let mut result = text.to_string();
    for (pattern, replacement) in POLITE_PATTERNS {
        // Case-insensitive replacement
        let lower = result.to_lowercase();
        if let Some(pos) = lower.find(&pattern.to_lowercase()) {
            let before = &result[..pos];
            let after = &result[pos + pattern.len()..];
            result = format!("{}{}{}", before, replacement, after);
        }
    }

    result.trim().to_string()
}

fn simplify_redundancy(text: &str) -> String {
    static REDUNDANT_PATTERNS: &[(&str, &str)] = &[
        ("in order to ", "to "),
        ("for the purpose of ", "to "),
        ("due to the fact that ", "because "),
        ("in the event that ", "if "),
        ("at this point in time ", "now "),
        ("in the near future ", "soon "),
        ("a large number of ", "many "),
        ("a small number of ", "few "),
        ("the majority of ", "most "),
        ("in the process of ", "currently "),
        ("on a daily basis ", "daily "),
        ("on a regular basis ", "regularly "),
        ("in a timely manner ", "quickly "),
        ("at the present time ", "now "),
        ("in the not too distant future ", "soon "),
        ("in light of the fact that ", "since "),
        ("given the fact that ", "since "),
        ("taking into consideration ", "considering "),
    ];

    let mut result = text.to_string();
    for (pattern, replacement) in REDUNDANT_PATTERNS {
        let lower = result.to_lowercase();
        if lower.contains(&pattern.to_lowercase()) {
            // Simple case-sensitive replace for first occurrence
            if let Some(pos) = result.to_lowercase().find(&pattern.to_lowercase()) {
                let before = &result[..pos];
                let after = &result[pos + pattern.len()..];
                result = format!("{}{}{}", before, replacement, after);
            }
        }
    }

    result.trim().to_string()
}

fn compress_instructions(text: &str) -> String {
    static INSTRUCTION_PATTERNS: &[(&str, &str)] = &[
        ("Make sure to ", ""),
        ("Ensure that you ", ""),
        ("Be sure to ", ""),
        ("Don't forget to ", ""),
        ("Remember to ", ""),
        ("It is important to ", ""),
        ("It is necessary to ", ""),
        ("You should ", ""),
        ("You need to ", ""),
        ("You must ", ""),
    ];

    let mut result = text.to_string();
    for (pattern, replacement) in INSTRUCTION_PATTERNS {
        let lower = result.to_lowercase();
        if let Some(pos) = lower.find(&pattern.to_lowercase()) {
            let before = &result[..pos];
            let after = &result[pos + pattern.len()..];
            result = format!("{}{}{}", before, replacement, after);
        }
    }

    result.trim().to_string()
}

fn compress_code_context(text: &str) -> String {
    // For code: remove natural language padding, keep technical terms
    let mut result = text.to_string();

    // Remove "please" before code-related words
    result = result.replace("please write", "write");
    result = result.replace("please create", "create");
    result = result.replace("please implement", "implement");
    result = result.replace("please fix", "fix");
    result = result.replace("please debug", "debug");

    // Remove "for me" and similar
    result = result.replace(" for me ", " ");
    result = result.replace(" for this project ", " ");

    result.trim().to_string()
}

fn compress_creative_context(text: &str) -> String {
    // For creative writing: keep more natural language but remove filler
    let mut result = text.to_string();
    result = result.replace("in a creative way", "creatively");
    result = result.replace("in an interesting way", "interestingly");
    result = result.replace("in a compelling way", "compellingly");
    result.trim().to_string()
}

fn compress_analysis_context(text: &str) -> String {
    // For analysis: be concise but precise
    let mut result = text.to_string();
    result = result.replace("please analyze", "analyze");
    result = result.replace("please provide an analysis of", "analyze");
    result = result.replace("give me a detailed analysis of", "analyze");
    result = result.replace("provide a comprehensive analysis of", "analyze");
    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_basic() {
        let compressor = PromptCompressor::medium();
        let (compressed, stats) =
            compressor.compress("Please kindly help me to understand this concept");
        assert!(stats.saved_tokens > 0, "Should save tokens");
        assert!(compressed.len() < "Please kindly help me to understand this concept".len());
    }

    #[test]
    fn test_compress_light() {
        let compressor = PromptCompressor::light();
        let (_, stats) = compressor.compress("Hello world this is a test");
        assert!(stats.original_tokens > 0);
    }

    #[test]
    fn test_compress_aggressive() {
        let compressor = PromptCompressor::aggressive();
        let (_compressed, stats) = compressor.compress(
            "I would like you to please in order to help me understand this concept due to the fact that I need to learn"
        );
        assert!(
            stats.saved_percent > 20.0,
            "Aggressive should save >20%, got {}%",
            stats.saved_percent
        );
    }

    #[test]
    fn test_filler_removal() {
        let compressor = PromptCompressor::medium();
        let (compressed, _) = compressor.compress("This is really very quite important");
        assert!(!compressed.contains("really"), "Should remove 'really'");
        assert!(!compressed.contains("very"), "Should remove 'very'");
        assert!(!compressed.contains("quite"), "Should remove 'quite'");
    }

    #[test]
    fn test_polite_phrase_removal() {
        let compressor = PromptCompressor::medium();
        let (compressed, _) = compressor.compress("Could you please explain this?");
        assert!(
            !compressed.contains("Could you please"),
            "Should remove polite phrase"
        );
    }

    #[test]
    fn test_estimate_tokens() {
        assert!(estimate_tokens("hello") >= 1);
        assert!(estimate_tokens("hello world") >= 2);
        assert!(estimate_tokens("this is a longer sentence with more words") >= 5);
    }

    #[test]
    fn test_compress_preserves_meaning() {
        let compressor = PromptCompressor::medium();
        let (compressed, _) = compressor.compress("Please explain how Rust ownership works");
        assert!(compressed.contains("explain"));
        assert!(compressed.contains("Rust"));
        assert!(compressed.contains("ownership"));
    }

    #[test]
    fn test_compress_code_context() {
        let compressor = PromptCompressor::medium();
        let (compressed, _) = compressor.compress_for(
            "Please write a function for me that sorts an array",
            "code programming",
        );
        assert!(compressed.contains("write"));
        assert!(compressed.contains("function"));
    }
}

use promptmaxx_core::{GitInfo, Pattern};

use crate::client::OllamaClient;
use crate::error::{OllamaError, Result};

/// Optimizes prompts before sending to Claude
pub struct PromptOptimizer {
    client: OllamaClient,
}

impl PromptOptimizer {
    pub fn new(client: OllamaClient) -> Self {
        Self { client }
    }

    /// Enhance a prompt using context and learned patterns
    pub async fn enhance(
        &self,
        original: &str,
        git_info: &GitInfo,
        recent_prompts: &[String],
        patterns: &[Pattern],
    ) -> Result<String> {
        let system_prompt = self.build_enhancement_prompt(git_info, recent_prompts, patterns);

        let full_prompt = format!(
            "{}\n\nUser's prompt to enhance:\n{}\n\nEnhanced prompt:",
            system_prompt, original
        );

        let response = self.client.generate(&full_prompt).await?;

        // Extract just the enhanced prompt (remove any explanation)
        let enhanced = self.extract_enhanced_prompt(&response, original);

        Ok(enhanced)
    }

    fn build_enhancement_prompt(
        &self,
        git_info: &GitInfo,
        recent_prompts: &[String],
        patterns: &[Pattern],
    ) -> String {
        let mut prompt = String::from(
            r#"You are a prompt enhancement assistant. Your job is to take a user's prompt for Claude Code (a coding assistant) and make it clearer, more specific, and more likely to get a good response.

Rules:
1. Keep the core intent intact
2. Add helpful context when it's missing
3. Make vague requests more specific
4. Don't add unnecessary verbosity
5. Output ONLY the enhanced prompt, nothing else

"#,
        );

        // Add git context
        if let Some(ref repo) = git_info.repo {
            prompt.push_str(&format!("Current repository: {}\n", repo));
        }
        if let Some(ref branch) = git_info.branch {
            prompt.push_str(&format!("Current branch: {}\n", branch));
        }

        // Add recent prompt context
        if !recent_prompts.is_empty() {
            prompt.push_str("\nRecent prompts (for context on user's work):\n");
            for (i, p) in recent_prompts.iter().take(3).enumerate() {
                let preview: String = p.chars().take(100).collect();
                prompt.push_str(&format!("{}. {}\n", i + 1, preview));
            }
        }

        // Add learned patterns
        let good_patterns: Vec<_> = patterns
            .iter()
            .filter(|p| p.success_rate() >= 0.6)
            .take(5)
            .collect();

        if !good_patterns.is_empty() {
            prompt.push_str("\nSuccessful patterns to apply:\n");
            for pattern in good_patterns {
                prompt.push_str(&format!("- {}\n", pattern.description));
            }
        }

        prompt
    }

    fn extract_enhanced_prompt(&self, response: &str, original: &str) -> String {
        // Try to find the enhanced prompt in the response
        let response = response.trim();

        // If response is empty or too short, return original
        if response.len() < original.len() / 2 {
            return original.to_string();
        }

        // Remove common prefixes that models add
        let cleaned = response
            .trim_start_matches("Enhanced prompt:")
            .trim_start_matches("Here's the enhanced prompt:")
            .trim_start_matches("Here is the enhanced prompt:")
            .trim();

        // If it's wrapped in quotes, remove them
        let cleaned = if cleaned.starts_with('"') && cleaned.ends_with('"') {
            &cleaned[1..cleaned.len() - 1]
        } else {
            cleaned
        };

        cleaned.to_string()
    }
}

/// Analyzes the effectiveness of a prompt/response pair
pub struct EffectivenessAnalyzer {
    client: OllamaClient,
}

/// Analysis result
pub struct AnalysisResult {
    pub score: f64,
    pub summary: String,
}

impl EffectivenessAnalyzer {
    pub fn new(client: OllamaClient) -> Self {
        Self { client }
    }

    /// Analyze a prompt/response pair for effectiveness
    pub async fn analyze(&self, original_prompt: &str, response: &str) -> Result<AnalysisResult> {
        // Truncate response if too long
        let response_preview: String = response.chars().take(2000).collect();

        let analysis_prompt = format!(
            r#"Analyze this coding assistant interaction and rate its effectiveness.

User's prompt:
{}

Assistant's response (truncated):
{}

Rate the interaction on these criteria:
1. Did the response address the user's request?
2. Was the response actionable and specific?
3. Was the response appropriately scoped (not too verbose)?

Output format (JSON only, no other text):
{{"score": 0.0-1.0, "summary": "one sentence summary"}}
"#,
            original_prompt, response_preview
        );

        let response = self.client.generate(&analysis_prompt).await?;

        // Parse the JSON response
        self.parse_analysis_response(&response)
    }

    fn parse_analysis_response(&self, response: &str) -> Result<AnalysisResult> {
        // Try to extract JSON from the response
        let response = response.trim();

        // Find JSON object in response
        let json_start = response.find('{');
        let json_end = response.rfind('}');

        match (json_start, json_end) {
            (Some(start), Some(end)) if start < end => {
                let json_str = &response[start..=end];

                #[derive(serde::Deserialize)]
                struct RawAnalysis {
                    score: f64,
                    summary: String,
                }

                let raw: RawAnalysis = serde_json::from_str(json_str)
                    .map_err(|e| OllamaError::Analysis(format!("Failed to parse: {}", e)))?;

                // Clamp score to valid range
                let score = raw.score.clamp(0.0, 1.0);

                Ok(AnalysisResult {
                    score,
                    summary: raw.summary,
                })
            }
            _ => {
                // Couldn't parse, return neutral score
                Ok(AnalysisResult {
                    score: 0.5,
                    summary: "Analysis could not be parsed".to_string(),
                })
            }
        }
    }
}

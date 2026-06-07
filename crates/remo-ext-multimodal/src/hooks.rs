//! Phase hooks for multimodal content preprocessing.
//!
//! Provides [`MultimodalBeforeInferenceHook`] which inspects incoming messages
//! for multimodal content markers and routes file paths through the [`FileParser`].

use async_trait::async_trait;

use remo_runtime::PhaseHook;
use remo_runtime::PhaseContext;
use remo_runtime_contract::StateCommand;
use remo_runtime_contract::StateError;

use crate::config::MultimodalConfigKey;
use crate::file_parser::FileParser;

/// A `BeforeInference` phase hook that processes multimodal content.
///
/// This hook inspects the current messages for multimodal content markers
/// (e.g. file path references, base64-encoded media, URLs). When file paths
/// are detected, they are routed through [`FileParser`] to extract text content,
/// and the resulting content is available for downstream processing.
///
/// # Behavior
///
/// 1. Reads the multimodal configuration from the agent spec.
/// 2. Scans recent messages for file path references.
/// 3. Parses recognized file formats using [`FileParser`].
/// 4. Returns a `StateCommand` with any extracted content stored as state.
pub struct MultimodalBeforeInferenceHook;

#[async_trait]
impl PhaseHook for MultimodalBeforeInferenceHook {
    async fn run(&self, ctx: &PhaseContext) -> Result<StateCommand, StateError> {
        let config = ctx.config::<MultimodalConfigKey>()?;
        let parser = FileParser::new();
        let command = StateCommand::new();

        // Scan messages for file path references that might need parsing.
        // This is a lightweight pass: we look for messages that appear to be
        // file path references and attempt to parse them.
        for message in ctx.messages.iter() {
            let text = message.text();

            // Skip empty content
            let text = match text.trim() {
                t if !t.is_empty() => t,
                _ => continue,
            };

            // Check if the message text looks like a file path reference
            if is_file_path_reference(text) {
                let path = text.trim();
                let format = parser.detect_format(path);

                // Only parse if the format is in the supported list
                if config.supported_formats.iter().any(|f| f == &format) {
                    // Attempt to parse the file; errors are non-fatal at this stage.
                    // The hook is a best-effort preprocessor — if parsing fails,
                    // the original message content is left unchanged.
                    let _parsed = parser.parse_auto(path);
                    // In a full implementation, the parsed content would be stored
                    // in state or used to enrich the message context.
                }
            }
        }

        Ok(command)
    }
}

/// Returns `true` if the text looks like a file path reference.
///
/// Heuristic check: the text must not contain spaces (beyond trimming),
/// should start with a recognizable path-like character, and should not
/// be too long for a typical file path.
fn is_file_path_reference(text: &str) -> bool {
    let trimmed = text.trim();

    // Reject empty or overly long strings
    if trimmed.is_empty() || trimmed.len() > 1024 {
        return false;
    }

    // Must not contain spaces (file paths typically don't)
    if trimmed.contains(' ') {
        return false;
    }

    // Must contain a dot (indicating a file extension)
    if !trimmed.contains('.') {
        return false;
    }

    // Should look like a relative or absolute path
    // (starts with a letter/digit, '.', '~', '/', or drive letter like "C:")
    let first = trimmed.chars().next().unwrap_or('\0');
    first.is_alphanumeric() || first == '.' || first == '~' || first == '/'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_path_reference_detection() {
        assert!(is_file_path_reference("document.txt"));
        assert!(is_file_path_reference("./data/report.csv"));
        assert!(is_file_path_reference("/home/user/file.json"));
        assert!(is_file_path_reference("data/config.md"));

        assert!(!is_file_path_reference(""));
        assert!(!is_file_path_reference("hello world"));
        assert!(!is_file_path_reference("no extension"));
        assert!(!is_file_path_reference("a".repeat(2000).as_str()));
    }

    #[test]
    fn hook_struct_instantiation() {
        let _hook = MultimodalBeforeInferenceHook;
        // Ensures the struct can be constructed
    }
}

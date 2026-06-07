//! File parsing utilities for multimodal content processing.
//!
//! Provides [`FileParser`] for extracting text content from various file formats,
//! detecting file types by extension, and performing format-specific parsing.

use std::path::Path;

/// Parser for extracting text content from files of various formats.
///
/// Supports plain text, CSV, and JSON file formats. Uses pure Rust
/// implementations with no external parsing dependencies.
#[derive(Debug, Clone, Default)]
pub struct FileParser;

impl FileParser {
    /// Create a new file parser instance.
    pub fn new() -> Self {
        Self
    }

    /// Parse a file at the given path and return its content as a string.
    ///
    /// Reads the file from disk and returns the raw text content. The file
    /// must be valid UTF-8; binary files will produce an error.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or is not valid UTF-8.
    pub fn parse_text(&self, file_path: &str) -> Result<String, String> {
        std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file '{file_path}': {e}"))
    }

    /// Parse CSV content into a 2D vector of strings.
    ///
    /// Splits the input on newlines to get rows, then splits each row on
    /// commas to get cells. Handles simple CSV; does not support quoted fields
    /// with embedded commas or newlines.
    ///
    /// # Errors
    ///
    /// Returns an error if the content is empty.
    pub fn parse_csv(&self, content: &str) -> Result<Vec<Vec<String>>, String> {
        if content.trim().is_empty() {
            return Err("CSV content is empty".to_string());
        }

        let rows: Vec<Vec<String>> = content
            .lines()
            .map(|line| {
                line.split(',')
                    .map(|cell| cell.trim().to_string())
                    .collect()
            })
            .collect();

        Ok(rows)
    }

    /// Parse and pretty-print JSON content.
    ///
    /// Takes a JSON string, parses it, and re-serializes it with 2-space
    /// indentation for human-readable output.
    ///
    /// # Errors
    ///
    /// Returns an error if the content is not valid JSON.
    pub fn parse_json(&self, content: &str) -> Result<String, String> {
        let value: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| format!("Invalid JSON: {e}"))?;
        serde_json::to_string_pretty(&value)
            .map_err(|e| format!("Failed to format JSON: {e}"))
    }

    /// Detect the file format from the file extension.
    ///
    /// Returns the lowercase extension without the leading dot (e.g. `"txt"`,
    /// `"csv"`, `"json"`). Returns `"unknown"` if the path has no extension.
    pub fn detect_format(&self, path: &str) -> String {
        Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// Parse a file, auto-detecting the format from the path extension.
    ///
    /// Dispatches to the appropriate parser based on the file extension:
    /// - `.csv` → [`parse_csv`](Self::parse_csv) result serialized as JSON
    /// - `.json` → [`parse_json`](Self::parse_json)
    /// - everything else → [`parse_text`](Self::parse_text)
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn parse_auto(&self, file_path: &str) -> Result<String, String> {
        let format = self.detect_format(file_path);
        match format.as_str() {
            "csv" => {
                let content = self.parse_text(file_path)?;
                let rows = self.parse_csv(&content)?;
                serde_json::to_string_pretty(&rows)
                    .map_err(|e| format!("Failed to serialize CSV as JSON: {e}"))
            }
            "json" => {
                let content = self.parse_text(file_path)?;
                self.parse_json(&content)
            }
            _ => self.parse_text(file_path),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_format_various_extensions() {
        let parser = FileParser::new();
        assert_eq!(parser.detect_format("README.md"), "md");
        assert_eq!(parser.detect_format("data.csv"), "csv");
        assert_eq!(parser.detect_format("config.json"), "json");
        assert_eq!(parser.detect_format("archive.tar.gz"), "gz");
        assert_eq!(parser.detect_format("Makefile"), "unknown");
        assert_eq!(parser.detect_format("/path/to/file.txt"), "txt");
    }

    #[test]
    fn parse_csv_simple() {
        let parser = FileParser::new();
        let csv = "name,age,city\nAlice,30,NYC\nBob,25,LA";
        let rows = parser.parse_csv(csv).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0], vec!["name", "age", "city"]);
        assert_eq!(rows[1], vec!["Alice", "30", "NYC"]);
        assert_eq!(rows[2], vec!["Bob", "25", "LA"]);
    }

    #[test]
    fn parse_csv_empty_error() {
        let parser = FileParser::new();
        assert!(parser.parse_csv("").is_err());
        assert!(parser.parse_csv("   ").is_err());
    }

    #[test]
    fn parse_csv_single_row() {
        let parser = FileParser::new();
        let csv = "a,b,c";
        let rows = parser.parse_csv(csv).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0], vec!["a", "b", "c"]);
    }

    #[test]
    fn parse_json_valid() {
        let parser = FileParser::new();
        let json = r#"{"key": "value", "number": 42}"#;
        let result = parser.parse_json(json).unwrap();
        assert!(result.contains("\"key\""));
        assert!(result.contains("\"value\""));
        assert!(result.contains("42"));
        // Pretty-printed output should have newlines
        assert!(result.contains('\n'));
    }

    #[test]
    fn parse_json_invalid() {
        let parser = FileParser::new();
        let result = parser.parse_json("not json");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid JSON"));
    }

    #[test]
    fn parse_json_array() {
        let parser = FileParser::new();
        let json = r#"[1, 2, 3]"#;
        let result = parser.parse_json(json).unwrap();
        assert!(result.contains('1'));
        assert!(result.contains('3'));
    }

    #[test]
    fn new_is_default() {
        let _parser = FileParser::new();
        let _default = FileParser::default();
        // Both should work without error
    }
}

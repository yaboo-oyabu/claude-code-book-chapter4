//! Markdown file parsing and serialization with YAML front matter.

use crate::error::TaskCtlError;
use serde::de::DeserializeOwned;
use serde::Serialize;

const FRONT_MATTER_DELIMITER: &str = "---";

/// Parse YAML front matter and markdown body from a string.
pub fn parse<T: DeserializeOwned>(content: &str, path: &str) -> Result<(T, String), TaskCtlError> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with(FRONT_MATTER_DELIMITER) {
        return Err(TaskCtlError::ParseError {
            path: path.to_string(),
            source: anyhow::anyhow!("Missing front matter delimiter"),
        });
    }

    // Find second delimiter
    let after_first = &trimmed[FRONT_MATTER_DELIMITER.len()..];
    let yaml_end = after_first
        .find(&format!("\n{FRONT_MATTER_DELIMITER}"))
        .ok_or_else(|| TaskCtlError::ParseError {
            path: path.to_string(),
            source: anyhow::anyhow!("Missing closing front matter delimiter"),
        })?;

    let yaml_str = &after_first[..yaml_end];
    let rest_start = yaml_end + 1 + FRONT_MATTER_DELIMITER.len();
    let body = if rest_start < after_first.len() {
        after_first[rest_start..]
            .trim_start_matches('\n')
            .to_string()
    } else {
        String::new()
    };

    let data: T = serde_yaml::from_str(yaml_str).map_err(|e| TaskCtlError::ParseError {
        path: path.to_string(),
        source: anyhow::Error::new(e),
    })?;

    Ok((data, body))
}

/// Serialize data as YAML front matter + markdown body.
pub fn serialize<T: Serialize>(data: &T, body: &str) -> Result<String, TaskCtlError> {
    let yaml = serde_yaml::to_string(data).map_err(|e| TaskCtlError::ParseError {
        path: String::new(),
        source: anyhow::Error::new(e),
    })?;

    let mut result = String::new();
    result.push_str(FRONT_MATTER_DELIMITER);
    result.push('\n');
    result.push_str(&yaml);
    result.push_str(FRONT_MATTER_DELIMITER);
    result.push('\n');

    if !body.is_empty() {
        result.push('\n');
        result.push_str(body);
        if !body.ends_with('\n') {
            result.push('\n');
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        id: u32,
        title: String,
    }

    #[test]
    fn parse_valid_front_matter() {
        let content = "---\nid: 1\ntitle: Test\n---\n\nSome body text.\n";
        let (data, body): (TestData, String) = parse(content, "test.md").unwrap();
        assert_eq!(data.id, 1);
        assert_eq!(data.title, "Test");
        assert_eq!(body, "Some body text.\n");
    }

    #[test]
    fn parse_no_body() {
        let content = "---\nid: 2\ntitle: NoBody\n---\n";
        let (data, body): (TestData, String) = parse(content, "test.md").unwrap();
        assert_eq!(data.id, 2);
        assert!(body.is_empty());
    }

    #[test]
    fn parse_missing_front_matter() {
        let content = "No front matter here.";
        let result: Result<(TestData, String), _> = parse(content, "test.md");
        assert!(result.is_err());
    }

    #[test]
    fn parse_missing_closing_delimiter() {
        let content = "---\nid: 1\ntitle: Test\n";
        let result: Result<(TestData, String), _> = parse(content, "test.md");
        assert!(result.is_err());
    }

    #[test]
    fn serialize_with_body() {
        let data = TestData {
            id: 1,
            title: "Test".to_string(),
        };
        let result = serialize(&data, "Body text.").unwrap();
        assert!(result.starts_with("---\n"));
        assert!(result.contains("id: 1"));
        assert!(result.contains("title: Test"));
        assert!(result.contains("---\n\nBody text.\n"));
    }

    #[test]
    fn serialize_without_body() {
        let data = TestData {
            id: 2,
            title: "NoBody".to_string(),
        };
        let result = serialize(&data, "").unwrap();
        assert!(result.starts_with("---\n"));
        assert!(result.ends_with("---\n"));
        assert!(!result.contains("\n\n"));
    }

    #[test]
    fn roundtrip() {
        let data = TestData {
            id: 42,
            title: "Roundtrip".to_string(),
        };
        let body = "Some notes here.\n";
        let serialized = serialize(&data, body).unwrap();
        let (parsed_data, parsed_body): (TestData, String) = parse(&serialized, "test.md").unwrap();
        assert_eq!(parsed_data, data);
        assert_eq!(parsed_body, body);
    }
}

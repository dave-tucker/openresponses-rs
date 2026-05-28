use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A function tool definition.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionTool {
    #[serde(rename = "type")]
    pub r#type: String, // "function"
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
    pub strict: Option<bool>,
}

/// Tool choice specification.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// "auto" | "required" | "none"
    Named(String),
    Specific(SpecificToolChoice),
}

/// A specific tool choice targeting a named function.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificToolChoice {
    #[serde(rename = "type")]
    pub r#type: String,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_choice_named() {
        let tc: ToolChoice = serde_json::from_str(r#""auto""#).unwrap();
        match tc {
            ToolChoice::Named(s) => assert_eq!(s, "auto"),
            _ => panic!("expected named"),
        }
    }

    #[test]
    fn test_tool_choice_specific() {
        let tc: ToolChoice = serde_json::from_str(r#"{"type":"function","name":"my_fn"}"#).unwrap();
        match tc {
            ToolChoice::Specific(s) => {
                assert_eq!(s.r#type, "function");
                assert_eq!(s.name, "my_fn");
            }
            _ => panic!("expected specific"),
        }
    }

    #[test]
    fn test_function_tool_roundtrip() {
        let json = r#"{"type":"function","name":"get_weather","description":"Get weather","parameters":{"type":"object"},"strict":null}"#;
        let ft: FunctionTool = serde_json::from_str(json).unwrap();
        assert_eq!(ft.name, "get_weather");
        let reserialized = serde_json::to_string(&ft).unwrap();
        let v1: serde_json::Value = serde_json::from_str(json).unwrap();
        let v2: serde_json::Value = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(v1, v2);
    }
}

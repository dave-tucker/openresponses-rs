use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::items::OutputItem;
use super::tools::{FunctionTool, ToolChoice};

/// Usage statistics for a response.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageResource {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub input_tokens_details: InputTokensDetails,
    pub output_tokens_details: OutputTokensDetails,
}

impl UsageResource {
    /// Construct usage with input and output token counts; all detail fields default to zero.
    pub fn new(input_tokens: u64, output_tokens: u64) -> Self {
        UsageResource {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
            input_tokens_details: InputTokensDetails { cached_tokens: 0 },
            output_tokens_details: OutputTokensDetails {
                reasoning_tokens: 0,
            },
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputTokensDetails {
    pub cached_tokens: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputTokensDetails {
    pub reasoning_tokens: u64,
}

/// Text format configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFormat {
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Text parameter.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextParam {
    pub format: TextFormat,
}

/// Reasoning configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningConfig {
    pub effort: Option<String>,
    pub summary: Option<String>,
}

/// Error detail within a response.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
}

/// The main response resource returned by the API.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseResource {
    pub id: String,
    pub object: String, // "response"
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub status: String,
    pub incomplete_details: Option<Value>,
    pub model: String,
    pub previous_response_id: Option<String>,
    pub instructions: Option<String>,
    pub output: Vec<OutputItem>,
    pub error: Option<ResponseError>,
    #[serde(default)]
    pub tools: Vec<FunctionTool>,
    pub tool_choice: ToolChoice,
    pub truncation: String,
    pub parallel_tool_calls: bool,
    pub text: TextParam,
    pub top_p: f64,
    pub presence_penalty: f64,
    pub frequency_penalty: f64,
    pub top_logprobs: u32,
    pub temperature: f64,
    pub reasoning: ReasoningConfig,
    pub usage: UsageResource,
    pub max_output_tokens: Option<u32>,
    pub max_tool_calls: Option<u32>,
    pub store: bool,
    pub background: bool,
    pub service_tier: String,
    #[serde(default)]
    pub metadata: serde_json::Map<String, Value>,
    pub safety_identifier: Option<String>,
    pub prompt_cache_key: Option<String>,
}

/// A compact resource returned by /responses/compact.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactResource {
    pub id: String,
    pub object: String, // "response.compaction"
    pub created_at: i64,
    pub status: String,
    pub output: Vec<OutputItem>,
    pub usage: UsageResource,
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL_RESPONSE_JSON: &str = r#"{
        "id": "resp_1",
        "object": "response",
        "created_at": 1000,
        "completed_at": 1001,
        "status": "completed",
        "incomplete_details": null,
        "model": "test",
        "previous_response_id": null,
        "instructions": null,
        "output": [],
        "error": null,
        "tools": [],
        "tool_choice": "auto",
        "truncation": "disabled",
        "parallel_tool_calls": false,
        "text": {"format": {"type": "text"}},
        "top_p": 1.0,
        "presence_penalty": 0.0,
        "frequency_penalty": 0.0,
        "top_logprobs": 0,
        "temperature": 1.0,
        "reasoning": {"effort": null, "summary": null},
        "usage": {
            "input_tokens": 1,
            "output_tokens": 2,
            "total_tokens": 3,
            "input_tokens_details": {"cached_tokens": 0},
            "output_tokens_details": {"reasoning_tokens": 0}
        },
        "max_output_tokens": null,
        "max_tool_calls": null,
        "store": true,
        "background": false,
        "service_tier": "default",
        "metadata": {},
        "safety_identifier": null,
        "prompt_cache_key": null
    }"#;

    #[test]
    fn test_response_resource_roundtrip() {
        let parsed: ResponseResource = serde_json::from_str(FULL_RESPONSE_JSON).unwrap();
        assert_eq!(parsed.id, "resp_1");
        assert_eq!(parsed.status, "completed");
        assert_eq!(parsed.model, "test");
        let reserialized = serde_json::to_string(&parsed).unwrap();
        let reparsed: serde_json::Value = serde_json::from_str(&reserialized).unwrap();
        let original: serde_json::Value = serde_json::from_str(FULL_RESPONSE_JSON).unwrap();
        assert_eq!(reparsed, original);
    }

    #[test]
    fn test_compact_resource() {
        let json = r#"{
            "id":"resp_c1","object":"response.compaction","created_at":1000,"status":"completed",
            "output":[{"type":"compaction","id":"cmp_1","status":"completed"}],
            "usage":{"input_tokens":1,"output_tokens":2,"total_tokens":3,
                     "input_tokens_details":{"cached_tokens":0},
                     "output_tokens_details":{"reasoning_tokens":0}}
        }"#;
        let cr: CompactResource = serde_json::from_str(json).unwrap();
        assert_eq!(cr.object, "response.compaction");
        assert_eq!(cr.output.len(), 1);
        // Re-serialize and check the type tag is present
        let serialized = serde_json::to_string(&cr).unwrap();
        let v: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(v["output"][0]["type"], "compaction");
    }
}

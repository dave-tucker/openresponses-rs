use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::items::ItemParam;
use super::response::{ReasoningConfig, TextParam};
use super::tools::{FunctionTool, ToolChoice};

/// A string or a list of input items.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrItems {
    String(String),
    Items(Vec<ItemParam>),
}

impl StringOrItems {
    /// Convert a plain string into a user message item list.
    pub fn into_items(self) -> Vec<ItemParam> {
        use super::items::{MessageContent, MessageItemParam, MessageRole};
        match self {
            StringOrItems::String(s) => vec![ItemParam::Message(MessageItemParam {
                role: MessageRole::User,
                content: MessageContent::String(s),
                phase: None,
                id: None,
                status: None,
            })],
            StringOrItems::Items(items) => items,
        }
    }
}

/// Reasoning parameters from the request.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningParam {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

impl From<ReasoningParam> for ReasoningConfig {
    fn from(p: ReasoningParam) -> Self {
        ReasoningConfig {
            effort: p.effort,
            summary: p.summary,
        }
    }
}

/// Body for POST /responses.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResponseBody {
    pub model: String,
    pub input: StringOrItems,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<FunctionTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Map<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
}

/// Body for POST /responses/compact.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactResponseMethodPublicBody {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<StringOrItems>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
}

/// WebSocket response create event (client → server).
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketResponseCreateEvent {
    #[serde(rename = "type")]
    pub r#type: String, // "response.create"
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<StringOrItems>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<FunctionTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Map<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,
}

impl WebSocketResponseCreateEvent {
    /// Convert this WebSocket event into a CreateResponseBody.
    pub fn into_create_body(self) -> CreateResponseBody {
        let input = self.input.unwrap_or_else(|| StringOrItems::Items(vec![]));
        CreateResponseBody {
            model: self.model,
            input,
            previous_response_id: self.previous_response_id,
            stream: self.stream,
            stream_options: None,
            tools: self.tools,
            tool_choice: self.tool_choice,
            parallel_tool_calls: self.parallel_tool_calls,
            temperature: self.temperature,
            top_p: self.top_p,
            presence_penalty: self.presence_penalty,
            frequency_penalty: self.frequency_penalty,
            max_output_tokens: self.max_output_tokens,
            max_tool_calls: self.max_tool_calls,
            reasoning: self.reasoning,
            text: self.text,
            background: self.background,
            store: self.store,
            instructions: self.instructions,
            truncation: self.truncation,
            metadata: self.metadata,
            safety_identifier: self.safety_identifier,
            prompt_cache_key: self.prompt_cache_key,
            top_logprobs: self.top_logprobs,
            service_tier: self.service_tier,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_body_string_input() {
        let json = r#"{"model":"gpt-4o-mini","input":"Hello"}"#;
        let body: CreateResponseBody = serde_json::from_str(json).unwrap();
        assert_eq!(body.model, "gpt-4o-mini");
        match &body.input {
            StringOrItems::String(s) => assert_eq!(s, "Hello"),
            _ => panic!("expected string"),
        }
    }

    #[test]
    fn test_create_body_array_input() {
        let json =
            r#"{"model":"gpt-4o-mini","input":[{"type":"message","role":"user","content":"Hi"}]}"#;
        let body: CreateResponseBody = serde_json::from_str(json).unwrap();
        match &body.input {
            StringOrItems::Items(items) => assert_eq!(items.len(), 1),
            _ => panic!("expected items"),
        }
    }

    #[test]
    fn test_compact_body_requires_model() {
        let json = r#"{"model":"gpt-4o"}"#;
        let body: CompactResponseMethodPublicBody = serde_json::from_str(json).unwrap();
        assert_eq!(body.model, "gpt-4o");
    }

    #[test]
    fn test_ws_event_roundtrip() {
        let json = r#"{"type":"response.create","model":"gpt-4o","input":"Hello","store":false}"#;
        let ev: WebSocketResponseCreateEvent = serde_json::from_str(json).unwrap();
        assert_eq!(ev.r#type, "response.create");
        assert_eq!(ev.store, Some(false));
    }
}

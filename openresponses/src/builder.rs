use std::time::{SystemTime, UNIX_EPOCH};

use uuid::Uuid;

use crate::types::{
    FunctionTool, MessageOutputItem, OutputContent, OutputItem, OutputTextContent, ReasoningConfig,
    ResponseResource, TextFormat, TextParam, ToolChoice, UsageResource,
};

// ---------------------------------------------------------------------------
// ID helpers
// ---------------------------------------------------------------------------

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Generate a new unique ID with the given prefix, e.g. `new_id("resp")` → `"resp_a1b2c3…"`.
pub fn new_id(prefix: &str) -> String {
    format!("{}_{}", prefix, Uuid::new_v4().simple())
}

/// Generate a response ID (`resp_…`).
pub fn new_response_id() -> String {
    new_id("resp")
}

/// Generate a message output item ID (`msg_…`).
pub fn new_message_id() -> String {
    new_id("msg")
}

/// Generate a function-call ID (`fc_…`).
pub fn new_function_call_id() -> String {
    new_id("fc")
}

/// Generate a call ID for a function-call output (`call_…`).
pub fn new_call_id() -> String {
    new_id("call")
}

// ---------------------------------------------------------------------------
// ResponseBuilder
// ---------------------------------------------------------------------------

/// Builder for [`ResponseResource`].
///
/// Sensible defaults are provided for every field so you only need to set what
/// matters for your backend.
///
/// ```rust,no_run
/// # use openresponses::builder::{ResponseBuilder, new_response_id};
/// # use openresponses::types::UsageResource;
/// let resp = ResponseBuilder::new("my-model")
///     .id(new_response_id())
///     .store(false)
///     .build_text("Hello!", UsageResource::new(10, 5));
/// ```
pub struct ResponseBuilder {
    pub id: String,
    pub model: String,
    pub created_at: i64,
    pub previous_response_id: Option<String>,
    pub instructions: Option<String>,
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
    pub max_output_tokens: Option<u32>,
    pub max_tool_calls: Option<u32>,
    pub store: bool,
    pub background: bool,
    pub service_tier: String,
    pub metadata: serde_json::Map<String, serde_json::Value>,
    pub safety_identifier: Option<String>,
    pub prompt_cache_key: Option<String>,
}

impl ResponseBuilder {
    /// Create a new builder for the given model with sensible defaults.
    pub fn new(model: impl Into<String>) -> Self {
        ResponseBuilder {
            id: new_response_id(),
            model: model.into(),
            created_at: unix_now(),
            previous_response_id: None,
            instructions: None,
            tools: vec![],
            tool_choice: ToolChoice::Named("auto".to_string()),
            truncation: "disabled".to_string(),
            parallel_tool_calls: false,
            text: TextParam {
                format: TextFormat {
                    r#type: "text".to_string(),
                },
            },
            top_p: 1.0,
            presence_penalty: 0.0,
            frequency_penalty: 0.0,
            top_logprobs: 0,
            temperature: 1.0,
            reasoning: ReasoningConfig {
                effort: None,
                summary: None,
            },
            max_output_tokens: None,
            max_tool_calls: None,
            store: false,
            background: false,
            service_tier: "default".to_string(),
            metadata: Default::default(),
            safety_identifier: None,
            prompt_cache_key: None,
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn created_at(mut self, ts: i64) -> Self {
        self.created_at = ts;
        self
    }

    pub fn previous_response_id(mut self, id: impl Into<String>) -> Self {
        self.previous_response_id = Some(id.into());
        self
    }

    pub fn instructions(mut self, s: impl Into<String>) -> Self {
        self.instructions = Some(s.into());
        self
    }

    pub fn tools(mut self, tools: Vec<FunctionTool>) -> Self {
        self.tools = tools;
        self
    }

    pub fn tool_choice(mut self, tc: ToolChoice) -> Self {
        self.tool_choice = tc;
        self
    }

    pub fn store(mut self, store: bool) -> Self {
        self.store = store;
        self
    }

    pub fn parallel_tool_calls(mut self, parallel_tool_calls: bool) -> Self {
        self.parallel_tool_calls = parallel_tool_calls;
        self
    }

    pub fn temperature(mut self, t: f64) -> Self {
        self.temperature = t;
        self
    }

    pub fn max_output_tokens(mut self, n: u32) -> Self {
        self.max_output_tokens = Some(n);
        self
    }

    pub fn prompt_cache_key(mut self, key: impl Into<String>) -> Self {
        self.prompt_cache_key = Some(key.into());
        self
    }

    /// Build a completed `ResponseResource` with the given output items and usage.
    pub fn build_with_output(
        self,
        output: Vec<OutputItem>,
        usage: UsageResource,
    ) -> ResponseResource {
        let now = unix_now();
        let reasoning = if self.reasoning.effort.is_none() && self.reasoning.summary.is_none() {
            None
        } else {
            Some(self.reasoning)
        };

        ResponseResource {
            id: self.id,
            object: "response".to_string(),
            created_at: self.created_at,
            completed_at: Some(now),
            status: "completed".to_string(),
            incomplete_details: None,
            model: self.model,
            previous_response_id: self.previous_response_id,
            instructions: self.instructions,
            output,
            error: None,
            tools: self.tools,
            tool_choice: self.tool_choice,
            truncation: self.truncation,
            parallel_tool_calls: self.parallel_tool_calls,
            text: self.text,
            top_p: self.top_p,
            presence_penalty: self.presence_penalty,
            frequency_penalty: self.frequency_penalty,
            top_logprobs: self.top_logprobs,
            temperature: self.temperature,
            reasoning,
            usage,
            max_output_tokens: self.max_output_tokens,
            max_tool_calls: self.max_tool_calls,
            store: self.store,
            background: self.background,
            service_tier: self.service_tier,
            metadata: self.metadata,
            safety_identifier: self.safety_identifier,
            prompt_cache_key: self.prompt_cache_key,
        }
    }

    /// Build a completed `ResponseResource` with a single assistant text message.
    pub fn build_text(self, text: impl Into<String>, usage: UsageResource) -> ResponseResource {
        let msg_id = new_message_id();
        let text = text.into();
        let output = vec![OutputItem::Message(MessageOutputItem {
            id: msg_id,
            role: "assistant".to_string(),
            content: vec![OutputContent::OutputText(OutputTextContent {
                text,
                annotations: vec![],
                logprobs: vec![],
            })],
            status: "completed".to_string(),
            phase: None,
        })];
        self.build_with_output(output, usage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let resp = ResponseBuilder::new("gpt-4o").build_text("hi", UsageResource::new(1, 2));
        assert_eq!(resp.model, "gpt-4o");
        assert_eq!(resp.status, "completed");
        assert!(resp.id.starts_with("resp_"));
        assert_eq!(resp.output.len(), 1);
        assert!(!resp.store);
        assert!(!resp.parallel_tool_calls);
        assert_eq!(resp.usage.input_tokens, 1);
        assert_eq!(resp.usage.output_tokens, 2);
    }

    #[test]
    fn test_builder_overrides() {
        let resp = ResponseBuilder::new("m")
            .id("resp_custom")
            .store(false)
            .temperature(0.5)
            .build_text("hi", UsageResource::new(0, 0));
        assert_eq!(resp.id, "resp_custom");
        assert!(!resp.store);
        assert!((resp.temperature - 0.5).abs() < f64::EPSILON);
    }
}

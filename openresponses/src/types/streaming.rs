use serde::{Deserialize, Serialize};

use super::items::{ContentPart, OutputItem, UrlCitation};
use super::response::ResponseResource;

// ---------------------------------------------------------------------------
// Shared event structures
// ---------------------------------------------------------------------------

/// A response lifecycle event (response.created, response.completed, etc.)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseLifecycleEvent {
    pub sequence_number: u64,
    pub response: ResponseResource,
}

/// An output item event (response.output_item.added / done).
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputItemEvent {
    pub sequence_number: u64,
    pub item_id: String,
    pub output_index: u32,
    pub item: OutputItem,
}

/// A content part event (deprecated but included for compliance).
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPartEvent {
    pub sequence_number: u64,
    pub item_id: String,
    pub output_index: u32,
    pub content_index: u32,
    pub part: ContentPart,
}

/// A text delta event.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDeltaEvent {
    pub sequence_number: u64,
    pub item_id: String,
    pub output_index: u32,
    pub content_index: u32,
    pub delta: String,
}

/// A text done event.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDoneEvent {
    pub sequence_number: u64,
    pub item_id: String,
    pub output_index: u32,
    pub content_index: u32,
    pub text: String,
}

/// An annotation added event.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationAddedEvent {
    pub sequence_number: u64,
    pub item_id: String,
    pub content_index: u32,
    pub annotation_index: u32,
    pub annotation: UrlCitation,
}

/// A function call arguments delta event.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallDeltaEvent {
    pub sequence_number: u64,
    pub item_id: String,
    pub call_id: String,
    pub output_index: u32,
    pub delta: String,
}

/// A function call arguments done event.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallDoneEvent {
    pub sequence_number: u64,
    pub item_id: String,
    pub call_id: String,
    pub output_index: u32,
    pub arguments: String,
}

/// A reasoning delta event.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningDeltaEvent {
    pub sequence_number: u64,
    pub item_id: String,
    pub output_index: u32,
    pub delta: String,
}

/// A reasoning done event.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningDoneEvent {
    pub sequence_number: u64,
    pub item_id: String,
    pub output_index: u32,
    pub text: String,
}

/// A reasoning summary part event.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningSummaryPartEvent {
    pub sequence_number: u64,
    pub item_id: String,
    pub output_index: u32,
    pub summary_index: u32,
}

/// A reasoning summary delta event.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningSummaryDeltaEvent {
    pub sequence_number: u64,
    pub item_id: String,
    pub output_index: u32,
    pub summary_index: u32,
    pub delta: String,
}

/// A reasoning summary done event.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningSummaryDoneEvent {
    pub sequence_number: u64,
    pub item_id: String,
    pub output_index: u32,
    pub summary_index: u32,
    pub text: String,
}

/// A refusal delta event.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefusalDeltaEvent {
    pub sequence_number: u64,
    pub item_id: String,
    pub output_index: u32,
    pub content_index: u32,
    pub delta: String,
}

/// A refusal done event.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefusalDoneEvent {
    pub sequence_number: u64,
    pub item_id: String,
    pub output_index: u32,
    pub content_index: u32,
    pub refusal: String,
}

/// An error event detail.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    pub param: Option<String>,
}

/// An error event sent in the stream.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub sequence_number: u64,
    pub status: u16,
    pub error: ErrorDetail,
}

// ---------------------------------------------------------------------------
// StreamEvent enum
// ---------------------------------------------------------------------------

/// All possible streaming events, tagged with `type`.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    #[serde(rename = "response.created")]
    ResponseCreated(ResponseLifecycleEvent),
    #[serde(rename = "response.queued")]
    ResponseQueued(ResponseLifecycleEvent),
    #[serde(rename = "response.in_progress")]
    ResponseInProgress(ResponseLifecycleEvent),
    #[serde(rename = "response.completed")]
    ResponseCompleted(ResponseLifecycleEvent),
    #[serde(rename = "response.failed")]
    ResponseFailed(ResponseLifecycleEvent),
    #[serde(rename = "response.incomplete")]
    ResponseIncomplete(ResponseLifecycleEvent),
    #[serde(rename = "response.output_item.added")]
    OutputItemAdded(OutputItemEvent),
    #[serde(rename = "response.output_item.done")]
    OutputItemDone(OutputItemEvent),
    #[serde(rename = "response.content_part.added")]
    ContentPartAdded(ContentPartEvent),
    #[serde(rename = "response.content_part.done")]
    ContentPartDone(ContentPartEvent),
    #[serde(rename = "response.output_text.delta")]
    OutputTextDelta(TextDeltaEvent),
    #[serde(rename = "response.output_text.done")]
    OutputTextDone(TextDoneEvent),
    #[serde(rename = "response.output_text.annotation.added")]
    OutputTextAnnotationAdded(AnnotationAddedEvent),
    #[serde(rename = "response.function_call_arguments.delta")]
    FunctionCallArgumentsDelta(FunctionCallDeltaEvent),
    #[serde(rename = "response.function_call_arguments.done")]
    FunctionCallArgumentsDone(FunctionCallDoneEvent),
    #[serde(rename = "response.reasoning.delta")]
    ReasoningDelta(ReasoningDeltaEvent),
    #[serde(rename = "response.reasoning.done")]
    ReasoningDone(ReasoningDoneEvent),
    #[serde(rename = "response.reasoning_summary_part.added")]
    ReasoningSummaryPartAdded(ReasoningSummaryPartEvent),
    #[serde(rename = "response.reasoning_summary_part.done")]
    ReasoningSummaryPartDone(ReasoningSummaryPartEvent),
    #[serde(rename = "response.reasoning_summary.delta")]
    ReasoningSummaryDelta(ReasoningSummaryDeltaEvent),
    #[serde(rename = "response.reasoning_summary.done")]
    ReasoningSummaryDone(ReasoningSummaryDoneEvent),
    #[serde(rename = "response.refusal.delta")]
    RefusalDelta(RefusalDeltaEvent),
    #[serde(rename = "response.refusal.done")]
    RefusalDone(RefusalDoneEvent),
    #[serde(rename = "error")]
    Error(ErrorEvent),
}

impl StreamEvent {
    /// Returns true if this event terminates the stream.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            StreamEvent::ResponseCompleted(_)
                | StreamEvent::ResponseFailed(_)
                | StreamEvent::ResponseIncomplete(_)
        )
    }
}

/// WebSocket error message (distinct from streaming ErrorEvent).
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsError {
    #[serde(rename = "type")]
    pub r#type: String, // "error"
    pub status: u16,
    pub error: WsErrorDetail,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsErrorDetail {
    pub code: String,
    pub message: String,
    pub param: Option<String>,
}

impl WsError {
    pub fn new(status: u16, code: &str, message: &str, param: Option<&str>) -> Self {
        WsError {
            r#type: "error".to_string(),
            status,
            error: WsErrorDetail {
                code: code.to_string(),
                message: message.to_string(),
                param: param.map(|s| s.to_string()),
            },
        }
    }
}

/// An outbound message from the WebSocket session.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum WsOutbound {
    Event(Box<StreamEvent>),
    Done,
    Error(WsError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_event_type_tag() {
        // Test that the type tag is correctly serialized
        use super::super::response::{
            InputTokensDetails, OutputTokensDetails, TextFormat, TextParam, UsageResource,
        };
        use super::super::tools::ToolChoice;

        let resp = ResponseResource {
            id: "resp_1".to_string(),
            object: "response".to_string(),
            created_at: 1000,
            completed_at: Some(1001),
            status: "in_progress".to_string(),
            incomplete_details: None,
            model: "test".to_string(),
            previous_response_id: None,
            instructions: None,
            output: vec![],
            error: None,
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
            reasoning: None,
            usage: UsageResource {
                input_tokens: 1,
                output_tokens: 2,
                total_tokens: 3,
                input_tokens_details: InputTokensDetails { cached_tokens: 0 },
                output_tokens_details: OutputTokensDetails {
                    reasoning_tokens: 0,
                },
            },
            max_output_tokens: None,
            max_tool_calls: None,
            store: true,
            background: false,
            service_tier: "default".to_string(),
            metadata: Default::default(),
            safety_identifier: None,
            prompt_cache_key: None,
        };

        let event = StreamEvent::ResponseCreated(ResponseLifecycleEvent {
            sequence_number: 0,
            response: resp,
        });

        let json = serde_json::to_string(&event).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["type"], "response.created");
        assert_eq!(v["sequence_number"], 0);
        assert!(!event.is_terminal());
    }

    #[test]
    fn test_ws_error_serialization() {
        let err = WsError::new(
            404,
            "previous_response_not_found",
            "Not found",
            Some("previous_response_id"),
        );
        let json = serde_json::to_string(&err).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["type"], "error");
        assert_eq!(v["status"], 404);
        assert_eq!(v["error"]["code"], "previous_response_not_found");
    }

    #[test]
    fn test_function_call_delta_event() {
        let ev = StreamEvent::FunctionCallArgumentsDelta(FunctionCallDeltaEvent {
            sequence_number: 3,
            item_id: "fc_1".to_string(),
            call_id: "call_abc".to_string(),
            output_index: 0,
            delta: "{\"".to_string(),
        });
        let json = serde_json::to_string(&ev).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["type"], "response.function_call_arguments.delta");
    }
}

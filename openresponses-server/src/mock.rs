use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use futures::stream;
use tokio::sync::RwLock;
use uuid::Uuid;

use openresponses::{
    handler::{ResponseOrStream, ResponsesHandler},
    types::{
        CompactResource, CompactResponseMethodPublicBody, CompactionOutputItem, CreateResponseBody,
        FunctionCallItem, InputTokensDetails, MessageOutputItem, OutputContent, OutputItem,
        OutputTextContent, OutputTokensDetails, ResponseResource, TextFormat,
        TextParam, ToolChoice, UsageResource,
    },
    websocket::events_for_response,
};

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn new_id(prefix: &str) -> String {
    format!("{}_{}", prefix, Uuid::new_v4().simple())
}

fn default_response_resource(
    id: String,
    model: String,
    output: Vec<OutputItem>,
    store: bool,
) -> ResponseResource {
    let now = now_secs();
    ResponseResource {
        id,
        object: "response".to_string(),
        created_at: now,
        completed_at: Some(now),
        status: "completed".to_string(),
        incomplete_details: None,
        model,
        previous_response_id: None,
        instructions: None,
        output,
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
        store,
        background: false,
        service_tier: "default".to_string(),
        metadata: Default::default(),
        safety_identifier: None,
        prompt_cache_key: None,
    }
}

/// The mock backend. Generates plausible responses without calling any real LLM.
pub struct MockHandler {
    /// Global store for store:true responses (keyed by response ID).
    store: Arc<RwLock<HashMap<String, ResponseResource>>>,
}

impl MockHandler {
    pub fn new() -> Self {
        MockHandler {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn generate_output(&self, req: &CreateResponseBody) -> Vec<OutputItem> {
        let has_tools = req.tools.as_ref().is_some_and(|t| !t.is_empty());
        if has_tools {
            let tool = &req.tools.as_ref().unwrap()[0];
            vec![OutputItem::FunctionCall(FunctionCallItem {
                id: new_id("fc"),
                call_id: new_id("call"),
                name: tool.name.clone(),
                arguments: "{}".to_string(),
                status: "completed".to_string(),
            })]
        } else {
            vec![OutputItem::Message(MessageOutputItem {
                id: new_id("msg"),
                role: "assistant".to_string(),
                content: vec![OutputContent::OutputText(OutputTextContent {
                    text: "Hello! I'm ready to help.".to_string(),
                    annotations: vec![],
                })],
                status: "completed".to_string(),
                phase: None,
            })]
        }
    }

    fn generate_response(&self, req: &CreateResponseBody) -> ResponseResource {
        let id = new_id("resp");
        let output = self.generate_output(req);
        let store = req.store.unwrap_or(false);
        let mut resp = default_response_resource(id, req.model.clone(), output, store);
        resp.previous_response_id = req.previous_response_id.clone();
        resp.instructions = req.instructions.clone();
        if let Some(tools) = &req.tools {
            resp.tools = tools.clone();
        }
        if let Some(tc) = &req.tool_choice {
            resp.tool_choice = tc.clone();
        }
        resp
    }
}

impl Default for MockHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Error type for MockHandler.
#[derive(Debug)]
pub struct MockError(pub String);

impl std::fmt::Display for MockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockError: {}", self.0)
    }
}

#[async_trait]
impl ResponsesHandler for MockHandler {
    type Error = MockError;

    async fn create_response(
        &self,
        req: CreateResponseBody,
        _auth: Option<String>,
    ) -> Result<ResponseOrStream, MockError> {
        let resp = self.generate_response(&req);
        let store = req.store.unwrap_or(false);

        // Store globally if store:true
        if store {
            self.store
                .write()
                .await
                .insert(resp.id.clone(), resp.clone());
        }

        if req.stream.unwrap_or(false) {
            let events = events_for_response(resp);
            let s = stream::iter(events);
            Ok(ResponseOrStream::Stream(Box::pin(s)))
        } else {
            Ok(ResponseOrStream::Response(Box::new(resp)))
        }
    }

    async fn compact_response(
        &self,
        _req: CompactResponseMethodPublicBody,
        _auth: Option<String>,
    ) -> Result<CompactResource, MockError> {
        let id = new_id("resp");
        let cmp_id = new_id("cmp");
        let encrypted_content = base64_encode(b"mock-compaction-content");
        Ok(CompactResource {
            id,
            object: "response.compaction".to_string(),
            created_at: now_secs(),
            status: "completed".to_string(),
            output: vec![OutputItem::Compaction(CompactionOutputItem {
                id: cmp_id,
                status: "completed".to_string(),
                encrypted_content: Some(encrypted_content),
            })],
            usage: UsageResource {
                input_tokens: 1,
                output_tokens: 2,
                total_tokens: 3,
                input_tokens_details: InputTokensDetails { cached_tokens: 0 },
                output_tokens_details: OutputTokensDetails {
                    reasoning_tokens: 0,
                },
            },
        })
    }

    async fn get_response(&self, id: &str) -> Option<ResponseResource> {
        self.store.read().await.get(id).cloned()
    }
}

fn base64_encode(data: &[u8]) -> String {
    // Simple base64 encoding without an external dependency
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((n >> 18) & 0x3f) as usize] as char);
        out.push(CHARS[((n >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARS[((n >> 6) & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARS[(n & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use openresponses::types::StringOrItems;

    fn make_req(model: &str, input: &str) -> CreateResponseBody {
        CreateResponseBody {
            model: model.to_string(),
            input: StringOrItems::String(input.to_string()),
            previous_response_id: None,
            stream: None,
            stream_options: None,
            tools: None,
            tool_choice: None,
            parallel_tool_calls: None,
            temperature: None,
            top_p: None,
            presence_penalty: None,
            frequency_penalty: None,
            max_output_tokens: None,
            max_tool_calls: None,
            reasoning: None,
            text: None,
            background: None,
            store: None,
            instructions: None,
            truncation: None,
            metadata: None,
            safety_identifier: None,
            prompt_cache_key: None,
            top_logprobs: None,
            service_tier: None,
        }
    }

    #[tokio::test]
    async fn test_basic_response() {
        let handler = MockHandler::new();
        let req = make_req("gpt-4o-mini", "Hello");
        let result = handler.create_response(req, None).await.unwrap();
        match result {
            ResponseOrStream::Response(resp) => {
                assert_eq!(resp.status, "completed");
                assert_eq!(resp.object, "response");
                assert!(!resp.output.is_empty());
            }
            _ => panic!("expected non-streaming response"),
        }
    }

    #[tokio::test]
    async fn test_streaming_response() {
        use futures::StreamExt;
        let handler = MockHandler::new();
        let mut req = make_req("gpt-4o-mini", "Hello");
        req.stream = Some(true);
        let result = handler.create_response(req, None).await.unwrap();
        match result {
            ResponseOrStream::Stream(mut s) => {
                let mut events = Vec::new();
                while let Some(ev) = s.next().await {
                    events.push(ev);
                }
                assert!(!events.is_empty());
                assert!(events.last().unwrap().is_terminal());
            }
            _ => panic!("expected streaming response"),
        }
    }

    #[tokio::test]
    async fn test_tool_calling_response() {
        use openresponses::types::FunctionTool;
        let handler = MockHandler::new();
        let mut req = make_req("gpt-4o-mini", "What's the weather?");
        req.tools = Some(vec![FunctionTool {
            r#type: "function".to_string(),
            name: "get_weather".to_string(),
            description: Some("Get weather".to_string()),
            parameters: None,
            strict: None,
        }]);
        let result = handler.create_response(req, None).await.unwrap();
        match result {
            ResponseOrStream::Response(resp) => {
                assert!(!resp.output.is_empty());
                assert!(matches!(resp.output[0], OutputItem::FunctionCall(_)));
            }
            _ => panic!("expected non-streaming response"),
        }
    }

    #[tokio::test]
    async fn test_compact_response() {
        let handler = MockHandler::new();
        let req = CompactResponseMethodPublicBody {
            model: "gpt-4o".to_string(),
            input: None,
            previous_response_id: None,
            instructions: None,
            prompt_cache_key: None,
        };
        let result = handler.compact_response(req, None).await.unwrap();
        assert_eq!(result.object, "response.compaction");
        assert_eq!(result.status, "completed");
        assert!(!result.output.is_empty());
        assert!(matches!(result.output[0], OutputItem::Compaction(_)));
    }

    #[tokio::test]
    async fn test_get_response_stored() {
        let handler = MockHandler::new();
        let mut req = make_req("gpt-4o-mini", "Store me");
        req.store = Some(true);
        let result = handler.create_response(req, None).await.unwrap();
        let resp_id = match result {
            ResponseOrStream::Response(r) => r.id.clone(),
            _ => panic!(),
        };
        let fetched = handler.get_response(&resp_id).await;
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().id, resp_id);
    }

    #[tokio::test]
    async fn test_get_response_not_stored() {
        let handler = MockHandler::new();
        let mut req = make_req("gpt-4o-mini", "Don't store me");
        req.store = Some(false);
        let _ = handler.create_response(req, None).await.unwrap();
        // store:false → not in global store
        let fetched = handler.get_response("any_id").await;
        assert!(fetched.is_none());
    }
}

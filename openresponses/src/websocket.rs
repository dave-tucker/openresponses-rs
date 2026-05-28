use std::collections::HashMap;
use std::sync::Arc;

use futures::StreamExt;

use crate::handler::{ResponseOrStream, ResponsesHandler};
use crate::types::{
    CreateResponseBody, ItemParam, ResponseResource, StreamEvent, StringOrItems,
    WebSocketResponseCreateEvent, WsError, WsOutbound,
};

/// A WebSocket session that manages per-connection state.
pub struct WsSession<H: ResponsesHandler> {
    handler: Arc<H>,
    auth: Option<String>,
    /// store:false responses, keyed by response ID.
    local_store: HashMap<String, ResponseResource>,
}

impl<H: ResponsesHandler> WsSession<H> {
    /// Create a new session.
    pub fn new(handler: Arc<H>, auth: Option<String>) -> Self {
        WsSession {
            handler,
            auth,
            local_store: HashMap::new(),
        }
    }

    /// Handle an incoming WebSocket `response.create` event.
    /// Returns a list of outbound messages (events + Done/Error).
    pub async fn handle_message(&mut self, event: WebSocketResponseCreateEvent) -> Vec<WsOutbound> {
        let store = event.store.unwrap_or(true);
        let previous_response_id = event.previous_response_id.clone();

        // Validate previous_response_id
        if let Some(ref prev_id) = previous_response_id {
            let prev_response = if store {
                // store:true — check global store
                self.handler.get_response(prev_id).await
            } else {
                // store:false — check local store
                self.local_store.get(prev_id).cloned()
            };

            if prev_response.is_none() {
                return vec![WsOutbound::Error(WsError::new(
                    404,
                    "previous_response_not_found",
                    &format!("Previous response '{}' not found.", prev_id),
                    Some("previous_response_id"),
                ))];
            }

            // Validate function_call_output items — call_ids must match function_calls
            // in the previous response output.
            if let Some(prev) = &prev_response
                && let Some(ref input) = event.input
            {
                let items = match input {
                    StringOrItems::Items(items) => items.as_slice(),
                    StringOrItems::String(_) => &[],
                };
                if let Err(err_msg) = validate_function_call_outputs(items, prev) {
                    // Evict from local store on failed validation
                    self.local_store.remove(prev_id);
                    return vec![WsOutbound::Error(WsError::new(
                        400,
                        "invalid_function_call_output",
                        &err_msg,
                        Some("input"),
                    ))];
                }
            }
        }

        // Build create body
        let create_body: CreateResponseBody = event.into_create_body();

        // Call handler
        let result = self
            .handler
            .create_response(create_body, self.auth.clone())
            .await;

        match result {
            Err(e) => {
                vec![WsOutbound::Error(WsError::new(
                    500,
                    "internal_error",
                    &format!("Handler error: {e}"),
                    None,
                ))]
            }
            Ok(ResponseOrStream::Response(resp)) => {
                let resp = *resp;
                // Store if store:false (local) — for store:true the handler stores globally
                if !store {
                    self.local_store.insert(resp.id.clone(), resp.clone());
                }
                // Emit synthetic streaming events for the response
                let events = events_for_response(resp);
                events
                    .into_iter()
                    .map(|e| WsOutbound::Event(Box::new(e)))
                    .collect()
            }
            Ok(ResponseOrStream::Stream(mut stream)) => {
                let mut out = Vec::new();
                let mut final_response: Option<ResponseResource> = None;
                while let Some(event) = stream.next().await {
                    // Capture the completed response from the terminal event
                    if let StreamEvent::ResponseCompleted(ref lc) = event {
                        final_response = Some(lc.response.clone());
                    }
                    let is_terminal = event.is_terminal();
                    out.push(WsOutbound::Event(Box::new(event)));
                    if is_terminal {
                        break;
                    }
                }
                // Store if store:false
                if !store && let Some(resp) = final_response {
                    self.local_store.insert(resp.id.clone(), resp);
                }
                out
            }
        }
    }
}

/// Check that all `function_call_output` items in `input` have call_ids that
/// appear in `previous.output` as `function_call` items.
fn validate_function_call_outputs(
    items: &[ItemParam],
    previous: &ResponseResource,
) -> Result<(), String> {
    use crate::types::OutputItem;

    let known_call_ids: Vec<&str> = previous
        .output
        .iter()
        .filter_map(|item| {
            if let OutputItem::FunctionCall(fc) = item {
                Some(fc.call_id.as_str())
            } else {
                None
            }
        })
        .collect();

    for item in items {
        if let ItemParam::FunctionCallOutput(fco) = item
            && !known_call_ids.contains(&fco.call_id.as_str())
        {
            return Err(format!(
                "function_call_output has call_id '{}' which does not match any function_call in the previous response.",
                fco.call_id
            ));
        }
    }
    Ok(())
}

/// Generate the standard streaming event sequence for a completed response.
pub fn events_for_response(resp: ResponseResource) -> Vec<StreamEvent> {
    use crate::types::{
        ContentPart, ContentPartEvent, FunctionCallDeltaEvent, FunctionCallDoneEvent,
        OutputItemEvent, ResponseLifecycleEvent, TextDeltaEvent, TextDoneEvent,
    };

    let mut events = Vec::new();
    let mut seq: u64 = 0;

    // Helper to bump sequence
    let mut next_seq = || {
        let s = seq;
        seq += 1;
        s
    };

    // 1. response.created (status=in_progress)
    let mut created_resp = resp.clone();
    created_resp.status = "in_progress".to_string();
    events.push(StreamEvent::ResponseCreated(ResponseLifecycleEvent {
        sequence_number: next_seq(),
        response: created_resp.clone(),
    }));

    // 2. response.in_progress
    events.push(StreamEvent::ResponseInProgress(ResponseLifecycleEvent {
        sequence_number: next_seq(),
        response: created_resp,
    }));

    // 3. Output items
    for (output_index, item) in resp.output.iter().enumerate() {
        let output_index = output_index as u32;
        let item_id = item.id().to_string();

        // output_item.added (in_progress)
        events.push(StreamEvent::OutputItemAdded(OutputItemEvent {
            sequence_number: next_seq(),
            item_id: item_id.clone(),
            output_index,
            item: item.with_status("in_progress"),
        }));

        match item {
            crate::types::OutputItem::Message(msg) => {
                // For each content part
                for (content_index, content) in msg.content.iter().enumerate() {
                    let content_index = content_index as u32;

                    // Build the ContentPart for events
                    let part = match content {
                        crate::types::OutputContent::OutputText(t) => ContentPart::OutputText {
                            text: t.text.clone(),
                            annotations: t.annotations.clone(),
                        },
                        crate::types::OutputContent::Refusal(r) => ContentPart::Refusal {
                            refusal: r.refusal.clone(),
                        },
                    };

                    // content_part.added
                    events.push(StreamEvent::ContentPartAdded(ContentPartEvent {
                        sequence_number: next_seq(),
                        item_id: item_id.clone(),
                        output_index,
                        content_index,
                        part: part.clone(),
                    }));

                    match content {
                        crate::types::OutputContent::OutputText(t) => {
                            let text = t.text.clone();
                            // output_text.delta
                            events.push(StreamEvent::OutputTextDelta(TextDeltaEvent {
                                sequence_number: next_seq(),
                                item_id: item_id.clone(),
                                output_index,
                                content_index,
                                delta: text.clone(),
                            }));
                            // output_text.done
                            events.push(StreamEvent::OutputTextDone(TextDoneEvent {
                                sequence_number: next_seq(),
                                item_id: item_id.clone(),
                                output_index,
                                content_index,
                                text: text.clone(),
                            }));
                        }
                        crate::types::OutputContent::Refusal(r) => {
                            let refusal = r.refusal.clone();
                            events.push(StreamEvent::RefusalDelta(
                                crate::types::RefusalDeltaEvent {
                                    sequence_number: next_seq(),
                                    item_id: item_id.clone(),
                                    output_index,
                                    content_index,
                                    delta: refusal.clone(),
                                },
                            ));
                            events.push(StreamEvent::RefusalDone(crate::types::RefusalDoneEvent {
                                sequence_number: next_seq(),
                                item_id: item_id.clone(),
                                output_index,
                                content_index,
                                refusal,
                            }));
                        }
                    }

                    // content_part.done
                    events.push(StreamEvent::ContentPartDone(ContentPartEvent {
                        sequence_number: next_seq(),
                        item_id: item_id.clone(),
                        output_index,
                        content_index,
                        part,
                    }));
                }
            }
            crate::types::OutputItem::FunctionCall(fc) => {
                // function_call_arguments.delta
                events.push(StreamEvent::FunctionCallArgumentsDelta(
                    FunctionCallDeltaEvent {
                        sequence_number: next_seq(),
                        item_id: item_id.clone(),
                        call_id: fc.call_id.clone(),
                        output_index,
                        delta: fc.arguments.clone(),
                    },
                ));
                // function_call_arguments.done
                events.push(StreamEvent::FunctionCallArgumentsDone(
                    FunctionCallDoneEvent {
                        sequence_number: next_seq(),
                        item_id: item_id.clone(),
                        call_id: fc.call_id.clone(),
                        output_index,
                        arguments: fc.arguments.clone(),
                    },
                ));
            }
            _ => {
                // Other item types: no inner events
            }
        }

        // output_item.done (completed)
        events.push(StreamEvent::OutputItemDone(OutputItemEvent {
            sequence_number: next_seq(),
            item_id: item_id.clone(),
            output_index,
            item: item.with_status("completed"),
        }));
    }

    // Final: response.completed
    let mut completed_resp = resp.clone();
    completed_resp.status = "completed".to_string();
    events.push(StreamEvent::ResponseCompleted(ResponseLifecycleEvent {
        sequence_number: next_seq(),
        response: completed_resp,
    }));

    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FunctionCallItem, OutputItem, ResponseResource};

    fn make_test_response_with_message(id: &str) -> ResponseResource {
        use crate::types::*;
        ResponseResource {
            id: id.to_string(),
            object: "response".to_string(),
            created_at: 1000,
            completed_at: Some(1001),
            status: "completed".to_string(),
            incomplete_details: None,
            model: "test".to_string(),
            previous_response_id: None,
            instructions: None,
            output: vec![OutputItem::Message(MessageOutputItem {
                id: "msg_1".to_string(),
                role: "assistant".to_string(),
                content: vec![OutputContent::OutputText(OutputTextContent {
                    text: "Hello!".to_string(),
                    annotations: vec![],
                })],
                status: "completed".to_string(),
                phase: None,
            })],
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
            reasoning: ReasoningConfig {
                effort: None,
                summary: None,
            },
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
            store: false,
            background: false,
            service_tier: "default".to_string(),
            metadata: Default::default(),
            safety_identifier: None,
            prompt_cache_key: None,
        }
    }

    #[test]
    fn test_events_for_text_response() {
        let resp = make_test_response_with_message("resp_1");
        let events = events_for_response(resp);

        // Should have: created, in_progress, output_item.added, content_part.added,
        // output_text.delta, output_text.done, content_part.done, output_item.done, completed
        assert!(events.len() >= 9);

        // First event is created
        assert!(matches!(events[0], StreamEvent::ResponseCreated(_)));
        // Second is in_progress
        assert!(matches!(events[1], StreamEvent::ResponseInProgress(_)));
        // Last event is completed
        assert!(matches!(
            events.last().unwrap(),
            StreamEvent::ResponseCompleted(_)
        ));

        // Last event must be terminal
        assert!(events.last().unwrap().is_terminal());
    }

    #[test]
    fn test_validate_function_call_outputs_ok() {
        use crate::types::{FunctionCallOutputItemParam, ItemParam};

        let mut resp = make_test_response_with_message("resp_1");
        resp.output = vec![OutputItem::FunctionCall(FunctionCallItem {
            id: "fc_1".to_string(),
            call_id: "call_abc".to_string(),
            name: "fn".to_string(),
            arguments: "{}".to_string(),
            status: "completed".to_string(),
        })];

        let items = vec![ItemParam::FunctionCallOutput(FunctionCallOutputItemParam {
            call_id: "call_abc".to_string(),
            output: "result".to_string(),
            id: None,
            status: None,
        })];

        assert!(validate_function_call_outputs(&items, &resp).is_ok());
    }

    #[test]
    fn test_validate_function_call_outputs_mismatch() {
        use crate::types::{FunctionCallOutputItemParam, ItemParam};

        let resp = make_test_response_with_message("resp_1");
        // resp has no function_call items

        let items = vec![ItemParam::FunctionCallOutput(FunctionCallOutputItemParam {
            call_id: "call_openresponses_missing".to_string(),
            output: "result".to_string(),
            id: None,
            status: None,
        })];

        assert!(validate_function_call_outputs(&items, &resp).is_err());
    }
}

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Content parts
// ---------------------------------------------------------------------------

/// URL citation annotation on output text.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlCitation {
    #[serde(rename = "type")]
    pub r#type: String, // "url_citation"
    pub start_index: u32,
    pub end_index: u32,
    pub url: String,
    pub title: String,
}

/// A content part that can appear in input or output messages.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    #[serde(rename = "input_text")]
    InputText { text: String },
    #[serde(rename = "input_image")]
    InputImage {
        #[serde(skip_serializing_if = "Option::is_none")]
        image_url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
    #[serde(rename = "input_file")]
    InputFile {
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_url: Option<String>,
    },
    #[serde(rename = "output_text")]
    OutputText {
        text: String,
        #[serde(default)]
        annotations: Vec<UrlCitation>,
    },
    #[serde(rename = "refusal")]
    Refusal { refusal: String },
    #[serde(rename = "summary_text")]
    SummaryText { text: String },
}

/// Content of a message item — either a plain string or an array of parts.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    String(String),
    Parts(Vec<ContentPart>),
}

// ---------------------------------------------------------------------------
// Input items (ItemParam)
// ---------------------------------------------------------------------------

/// Role for a message input item.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    System,
    Assistant,
    Developer,
}

/// A message input item.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageItemParam {
    pub role: MessageRole,
    pub content: MessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// A function call input item.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallItemParam {
    pub call_id: String,
    pub name: String,
    pub arguments: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// A function call output input item.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallOutputItemParam {
    pub call_id: String,
    pub output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// A reasoning input item.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningItemParam {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub summary: Vec<SummaryText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_content: Option<String>,
}

/// A compaction input item.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionItemParam {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub encrypted_content: String,
}

/// An item reference input item.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemReferenceParam {
    pub id: String,
}

/// Summary text within a reasoning item.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryText {
    #[serde(rename = "type")]
    pub r#type: String, // "summary_text"
    pub text: String,
}

/// An input item parameter — discriminated by the `type` field.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ItemParam {
    #[serde(rename = "message")]
    Message(MessageItemParam),
    #[serde(rename = "function_call")]
    FunctionCall(FunctionCallItemParam),
    #[serde(rename = "function_call_output")]
    FunctionCallOutput(FunctionCallOutputItemParam),
    #[serde(rename = "reasoning")]
    Reasoning(ReasoningItemParam),
    #[serde(rename = "compaction")]
    Compaction(CompactionItemParam),
    #[serde(rename = "item_reference")]
    ItemReference(ItemReferenceParam),
}

// ---------------------------------------------------------------------------
// Output items (in ResponseResource)
// ---------------------------------------------------------------------------

/// Text content in an output message.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputTextContent {
    pub text: String,
    #[serde(default)]
    pub annotations: Vec<UrlCitation>,
}

/// Refusal content in an output message.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefusalContent {
    pub refusal: String,
}

/// Content in an output message item.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OutputContent {
    #[serde(rename = "output_text")]
    OutputText(OutputTextContent),
    #[serde(rename = "refusal")]
    Refusal(RefusalContent),
}

/// A message output item.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageOutputItem {
    pub id: String,
    pub role: String,
    pub content: Vec<OutputContent>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
}

/// A function call output item.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallItem {
    pub id: String,
    pub call_id: String,
    pub name: String,
    pub arguments: String,
    pub status: String,
}

/// A function call output (result) output item.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallOutputItem {
    pub id: String,
    pub call_id: String,
    pub output: String,
    pub status: String,
}

/// A reasoning output item.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningOutputItem {
    pub id: String,
    pub summary: Vec<SummaryText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_content: Option<String>,
    pub status: String,
}

/// A compaction output item.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionOutputItem {
    pub id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_content: Option<String>,
}

/// An output item — discriminated by the `type` field.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OutputItem {
    #[serde(rename = "message")]
    Message(MessageOutputItem),
    #[serde(rename = "function_call")]
    FunctionCall(FunctionCallItem),
    #[serde(rename = "function_call_output")]
    FunctionCallOutput(FunctionCallOutputItem),
    #[serde(rename = "reasoning")]
    Reasoning(ReasoningOutputItem),
    #[serde(rename = "compaction")]
    Compaction(CompactionOutputItem),
}

impl OutputItem {
    pub fn id(&self) -> &str {
        match self {
            OutputItem::Message(m) => &m.id,
            OutputItem::FunctionCall(f) => &f.id,
            OutputItem::FunctionCallOutput(f) => &f.id,
            OutputItem::Reasoning(r) => &r.id,
            OutputItem::Compaction(c) => &c.id,
        }
    }

    pub fn item_type(&self) -> &str {
        match self {
            OutputItem::Message(_) => "message",
            OutputItem::FunctionCall(_) => "function_call",
            OutputItem::FunctionCallOutput(_) => "function_call_output",
            OutputItem::Reasoning(_) => "reasoning",
            OutputItem::Compaction(_) => "compaction",
        }
    }

    /// Return a version of the item with `status` set to the given value.
    pub fn with_status(&self, status: &str) -> Self {
        match self {
            OutputItem::Message(m) => OutputItem::Message(MessageOutputItem {
                status: status.to_string(),
                ..m.clone()
            }),
            OutputItem::FunctionCall(f) => OutputItem::FunctionCall(FunctionCallItem {
                status: status.to_string(),
                ..f.clone()
            }),
            OutputItem::FunctionCallOutput(f) => {
                OutputItem::FunctionCallOutput(FunctionCallOutputItem {
                    status: status.to_string(),
                    ..f.clone()
                })
            }
            OutputItem::Reasoning(r) => OutputItem::Reasoning(ReasoningOutputItem {
                status: status.to_string(),
                ..r.clone()
            }),
            OutputItem::Compaction(c) => OutputItem::Compaction(CompactionOutputItem {
                status: status.to_string(),
                ..c.clone()
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_param_message_string_content() {
        let json = r#"{"type":"message","role":"user","content":"Hello"}"#;
        let item: ItemParam = serde_json::from_str(json).unwrap();
        match item {
            ItemParam::Message(m) => {
                matches!(m.role, MessageRole::User);
                matches!(m.content, MessageContent::String(_));
            }
            _ => panic!("expected message"),
        }
    }

    #[test]
    fn test_item_param_function_call() {
        let json = r#"{"type":"function_call","call_id":"call_123","name":"get_weather","arguments":"{}"}"#;
        let item: ItemParam = serde_json::from_str(json).unwrap();
        match item {
            ItemParam::FunctionCall(f) => {
                assert_eq!(f.call_id, "call_123");
                assert_eq!(f.name, "get_weather");
            }
            _ => panic!("expected function_call"),
        }
    }

    #[test]
    fn test_item_param_function_call_output() {
        let json = r#"{"type":"function_call_output","call_id":"call_123","output":"sunny"}"#;
        let item: ItemParam = serde_json::from_str(json).unwrap();
        match item {
            ItemParam::FunctionCallOutput(f) => {
                assert_eq!(f.call_id, "call_123");
                assert_eq!(f.output, "sunny");
            }
            _ => panic!("expected function_call_output"),
        }
    }

    #[test]
    fn test_output_item_message() {
        let json = r#"{"type":"message","id":"msg_1","role":"assistant","content":[{"type":"output_text","text":"Hello","annotations":[]}],"status":"completed"}"#;
        let item: OutputItem = serde_json::from_str(json).unwrap();
        match &item {
            OutputItem::Message(m) => {
                assert_eq!(m.id, "msg_1");
                assert_eq!(m.status, "completed");
            }
            _ => panic!("expected message"),
        }
        // roundtrip — serialize and re-parse as Value for comparison
        let reserialized = serde_json::to_string(&item).unwrap();
        let v2: serde_json::Value = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(v2["type"], "message");
        assert_eq!(v2["id"], "msg_1");
        assert_eq!(v2["content"][0]["type"], "output_text");
    }

    #[test]
    fn test_output_item_function_call() {
        let json = r#"{"type":"function_call","id":"fc_1","call_id":"call_abc","name":"fn","arguments":"{}","status":"completed"}"#;
        let item: OutputItem = serde_json::from_str(json).unwrap();
        match &item {
            OutputItem::FunctionCall(f) => {
                assert_eq!(f.call_id, "call_abc");
            }
            _ => panic!("expected function_call"),
        }
        let reserialized = serde_json::to_string(&item).unwrap();
        let v: serde_json::Value = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(v["type"], "function_call");
    }
}

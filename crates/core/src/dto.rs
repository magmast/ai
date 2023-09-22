use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionResponseMessage, Role};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub enum Message {
    System(String),
    User(String),
    Function { name: String, content: String },
    Assistant(AssistantMessage),
}

impl Message {
    pub fn is_system(&self) -> bool {
        if let Self::System(_) = &self {
            true
        } else {
            false
        }
    }
}

impl Into<ChatCompletionRequestMessage> for Message {
    fn into(self) -> ChatCompletionRequestMessage {
        match self {
            Self::System(content) => ChatCompletionRequestMessage {
                role: Role::System,
                content: Some(content),
                ..Default::default()
            },
            Self::User(content) => ChatCompletionRequestMessage {
                role: Role::User,
                content: Some(content),
                ..Default::default()
            },
            Self::Function { name, content } => ChatCompletionRequestMessage {
                role: Role::Function,
                content: Some(content),
                name: Some(name),
                ..Default::default()
            },
            Self::Assistant(message) => message.into(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub enum AssistantMessage {
    Content(String),
    FunctionCall(FunctionCall),
}

impl Into<ChatCompletionRequestMessage> for AssistantMessage {
    fn into(self) -> ChatCompletionRequestMessage {
        let (content, fc) = match self {
            Self::Content(content) => (Some(content), None),
            Self::FunctionCall(fc) => (None, Some(fc)),
        };

        ChatCompletionRequestMessage {
            role: Role::Assistant,
            content: content,
            name: None,
            function_call: fc.map(Into::into),
        }
    }
}

impl From<ChatCompletionResponseMessage> for AssistantMessage {
    fn from(value: ChatCompletionResponseMessage) -> Self {
        if value.role != Role::Assistant {
            panic!("Unexpected role: {:?}", value.role);
        }

        match (value.content, value.function_call) {
            (Some(content), None) => AssistantMessage::Content(content),
            (content, Some(function_call)) => AssistantMessage::FunctionCall(FunctionCall {
                content,
                name: function_call.name,
                args: function_call.arguments,
            }),
            _ => panic!("Unexpected assistant message"),
        }
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub struct FunctionCall {
    pub name: String,
    pub content: Option<String>,
    pub args: String,
}

impl Into<async_openai::types::FunctionCall> for FunctionCall {
    fn into(self) -> async_openai::types::FunctionCall {
        async_openai::types::FunctionCall {
            name: self.name.to_string(),
            arguments: self.args.to_string(),
        }
    }
}

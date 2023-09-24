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
        matches!(self, Self::System(_))
    }
}

impl From<Message> for ChatCompletionRequestMessage {
    fn from(message: Message) -> Self {
        match message {
            Message::System(content) => ChatCompletionRequestMessage {
                role: Role::System,
                content: Some(content),
                ..Default::default()
            },
            Message::User(content) => ChatCompletionRequestMessage {
                role: Role::User,
                content: Some(content),
                ..Default::default()
            },
            Message::Function { name, content } => ChatCompletionRequestMessage {
                role: Role::Function,
                content: Some(content),
                name: Some(name),
                ..Default::default()
            },
            Message::Assistant(message) => message.into(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub enum AssistantMessage {
    Content(String),
    FunctionCall(FunctionCall),
}

impl From<AssistantMessage> for ChatCompletionRequestMessage {
    fn from(assitant_message: AssistantMessage) -> ChatCompletionRequestMessage {
        let (content, fc) = match assitant_message {
            AssistantMessage::Content(content) => (Some(content), None),
            AssistantMessage::FunctionCall(fc) => (None, Some(fc)),
        };

        ChatCompletionRequestMessage {
            role: Role::Assistant,
            content,
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

impl From<FunctionCall> for async_openai::types::FunctionCall {
    fn from(fc: FunctionCall) -> async_openai::types::FunctionCall {
        async_openai::types::FunctionCall {
            name: fc.name.to_string(),
            arguments: fc.args.to_string(),
        }
    }
}

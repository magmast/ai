pub mod dto;
pub mod funcs;
pub mod history;
pub mod utils;

use async_openai::{
    config::OpenAIConfig,
    error::OpenAIError,
    types::{ChatCompletionFunctions, CreateChatCompletionRequest},
    Client,
};
use async_trait::async_trait;
use dto::{AssistantMessage, Message};
use funcs::{Function, FunctionDeclaration};
use history::{History, NoopHistory};
use serde_json::json;
use tracing::debug;

const DEFAULT_HISTORY_LIMIT: usize = 15;

pub struct Chat<P, H>
where
    P: Platform,
    H: History + Send + Sync,
{
    platform: P,
    system_message: Option<String>,
    functions: Vec<Function>,
    history: Cache<H>,
    /// Maximum number of messages in history.
    history_limit: usize,
}

impl<P, H> Default for Chat<P, H>
where
    P: Platform + Default,
    H: History + Send + Sync + Default,
{
    fn default() -> Self {
        Self {
            platform: P::default(),
            history_limit: DEFAULT_HISTORY_LIMIT,
            history: Cache::from(H::default()),
            functions: vec![],
            system_message: None,
        }
    }
}

impl<P, H> From<H> for Chat<P, H>
where
    P: Platform + Default,
    H: History + Send + Sync,
{
    fn from(history: H) -> Self {
        Self {
            history: Cache::from(history),
            platform: P::default(),
            history_limit: DEFAULT_HISTORY_LIMIT,
            functions: vec![],
            system_message: None,
        }
    }
}

impl Chat<OpenAiPlatform, NoopHistory> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<P, H> Chat<P, H>
where
    P: Platform,
    H: History + Send + Sync,
{
    pub fn function(&mut self, fb: impl Into<Function>) -> &mut Self {
        self.functions.push(fb.into());
        self
    }

    pub fn system_message(&mut self, system_message: Option<String>) -> &mut Self {
        self.system_message = system_message;
        self
    }

    pub async fn send(
        &mut self,
        message: impl ToString,
    ) -> Result<String, SendError<P::Err, H::Err>> {
        let messages = self
            .build_messages(message)
            .await
            .map_err(SendError::ReadHistory)?;

        let (content, messages) = self.send_impl(messages).await?;

        let messages = messages
            .into_iter()
            .filter(|message| !message.is_system())
            .collect::<Vec<_>>();

        let messages_len = messages.len();

        let messages = if messages_len > self.history_limit {
            messages
                .into_iter()
                .skip(self.history_limit.abs_diff(messages_len))
                .collect()
        } else {
            messages
        };

        self.history
            .write(messages)
            .await
            .map_err(SendError::WriteHistory)?;

        Ok(content)
    }

    async fn send_impl(
        &mut self,
        mut messages: Vec<Message>,
    ) -> Result<(String, Vec<Message>), SendError<P::Err, H::Err>> {
        for _ in 0..10 {
            let completion = self
                .platform
                .create_completion(
                    messages.clone(),
                    self.functions.iter().map(Into::into).collect::<Vec<_>>(),
                )
                .await?;

            let (name, result) = match completion {
                AssistantMessage::Content(content) => {
                    messages.push(Message::Assistant(AssistantMessage::Content(
                        content.clone(),
                    )));
                    return Ok((content, messages));
                }
                AssistantMessage::FunctionCall(fc) => {
                    messages.push(Message::Assistant(AssistantMessage::FunctionCall(
                        fc.clone(),
                    )));

                    if let Some(content) = fc.content {
                        println!("{content}");
                    }

                    let result = match self.execute_function(&fc.name, &fc.args).await {
                        Ok(result) => result,
                        Err(err) => serde_json::to_string(&json!({
                            "error": err.to_string(),
                        }))
                        .unwrap(),
                    };

                    (fc.name, result)
                }
            };

            let message = Message::Function {
                name,
                content: result,
            };

            messages.push(message);
        }

        Err(SendError::TooManyFunctionCalls)
    }

    async fn execute_function(
        &mut self,
        name: &str,
        args: &str,
    ) -> Result<String, ExecuteFunctionError> {
        let function = self
            .functions
            .iter_mut()
            .find(|f| f.name == name)
            .ok_or_else(|| ExecuteFunctionError::FunctionNotFound(name.to_string()))?;

        Ok((function.execute)(args).await)
    }

    async fn build_messages(
        &mut self,
        user_message: impl ToString,
    ) -> Result<Vec<Message>, H::Err> {
        let history = self.history.read().await?;
        let mut messages =
            Vec::with_capacity(history.len() + if self.system_message.is_some() { 2 } else { 1 });

        if let Some(system_message) = &self.system_message {
            messages.push(Message::System(system_message.clone()));
        }
        messages.extend(history.clone());
        messages.push(Message::User(user_message.to_string()));

        Ok(messages)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SendError<P, H> {
    #[error("Platform error: {0}")]
    Platform(#[from] P),

    #[error("Failed to read history: {0}")]
    ReadHistory(H),

    #[error("Failed to write history: {0}")]
    WriteHistory(H),

    #[error("To many function calls")]
    TooManyFunctionCalls,
}

#[derive(Debug, thiserror::Error)]
enum ExecuteFunctionError {
    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(#[from] anyhow::Error),
}

#[async_trait]
pub trait Platform {
    type Err;

    async fn create_completion<M>(
        &self,
        messages: M,
        functions: Vec<FunctionDeclaration>,
    ) -> Result<AssistantMessage, Self::Err>
    where
        M: IntoIterator<Item = Message> + Send,
        M::IntoIter: Send;
}

pub struct OpenAiPlatform {
    client: Client<OpenAIConfig>,
}

#[async_trait]
impl Platform for OpenAiPlatform {
    type Err = OpenAIError;

    async fn create_completion<M>(
        &self,
        messages: M,
        functions: Vec<FunctionDeclaration>,
    ) -> Result<AssistantMessage, Self::Err>
    where
        M: IntoIterator<Item = Message> + Send,
        M::IntoIter: Send,
    {
        let request = CreateChatCompletionRequest {
            model: "gpt-3.5-turbo-16k".to_string(),
            functions: Some(
                functions
                    .into_iter()
                    .map(|f| ChatCompletionFunctions {
                        name: f.name,
                        description: f.description,
                        parameters: Some(f.args),
                    })
                    .collect(),
            ),
            messages: messages.into_iter().map(|message| message.into()).collect(),
            ..Default::default()
        };

        debug!(?request, "Sending ChatGPT request.");

        let mut result = self.client.chat().create(request).await?;

        Ok(result.choices.swap_remove(0).message.into())
    }
}

impl Default for OpenAiPlatform {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

/// Stands between `Chat` and `History` and caches messages to not read them all the time.
struct Cache<H: History> {
    history: H,
    messages: Option<Vec<Message>>,
}

impl<H: History> From<H> for Cache<H> {
    fn from(history: H) -> Self {
        Self {
            history,
            messages: None,
        }
    }
}

impl<H: History> Cache<H> {
    async fn read(&mut self) -> Result<&Vec<Message>, H::Err> {
        if self.messages.is_none() {
            let messages = self.history.read().await?;
            self.messages = Some(messages.clone());
        }

        Ok(self.messages.as_ref().unwrap())
    }

    async fn write(&mut self, messages: Vec<Message>) -> Result<(), H::Err> {
        self.history.write(&messages).await?;
        self.messages = Some(messages);
        Ok(())
    }
}

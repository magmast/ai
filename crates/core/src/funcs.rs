use std::{marker::PhantomData, sync::Arc};

use anyhow::Context;
use futures::{future::BoxFuture, Future, FutureExt};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::sync::Mutex;

pub struct FunctionDeclaration {
    pub name: String,
    pub description: Option<String>,
    pub args: Option<Value>,
}

pub(crate) struct Function {
    pub name: String,
    pub description: Option<String>,
    pub args: Option<Value>,
    pub execute: Box<dyn FnMut(String) -> BoxFuture<'static, Result<String, anyhow::Error>>>,
}

impl Function {
    pub fn to_declaration(&self) -> FunctionDeclaration {
        FunctionDeclaration {
            name: self.name.clone(),
            description: self.description.clone(),
            args: self.args.clone(),
        }
    }
}

pub trait FunctionArguments {
    fn json_schema() -> Option<Value>;
}

pub struct FunctionBuilder<Func, Args, Fut>
where
    Func: FnMut(Args) -> Fut + Send + 'static,
    Args: FunctionArguments + DeserializeOwned + Send,
    Fut: Future<Output = Result<String, anyhow::Error>> + Send,
{
    _args: PhantomData<Args>,
    name: Option<String>,
    description: Option<String>,
    execute: Option<Arc<Mutex<Func>>>,
}

impl<Func, Args, Fut> FunctionBuilder<Func, Args, Fut>
where
    Func: FnMut(Args) -> Fut + Send + Sync + 'static,
    Args: FunctionArguments + DeserializeOwned + Send,
    Fut: Future<Output = Result<String, anyhow::Error>> + Send,
{
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    pub fn execute(mut self, execute: Func) -> Self {
        self.execute = Some(Arc::new(Mutex::new(execute)));
        self
    }
}

impl<Func, Args, Fut> Into<Function> for FunctionBuilder<Func, Args, Fut>
where
    Func: FnMut(Args) -> Fut + Send + 'static,
    Args: FunctionArguments + DeserializeOwned + Send,
    Fut: Future<Output = Result<String, anyhow::Error>> + Send,
{
    fn into(self) -> Function {
        assert!(self.name.is_some(), "Name is required");
        assert!(self.execute.is_some(), "Execute is required");

        Function {
            name: self.name.unwrap(),
            description: self.description,
            args: Args::json_schema(),
            execute: Box::new(move |args| {
                let execute = Arc::clone(self.execute.as_ref().unwrap());

                async move {
                    let args = serde_json::from_str(&args).unwrap();
                    (execute.lock().await)(args)
                        .await
                        .context("Execution failed.")
                }
                .boxed()
            }),
        }
    }
}

impl<Func, Args, Fut> Default for FunctionBuilder<Func, Args, Fut>
where
    Func: FnMut(Args) -> Fut + Send + 'static,
    Args: FunctionArguments + DeserializeOwned + Send,
    Fut: Future<Output = Result<String, anyhow::Error>> + Send,
{
    fn default() -> Self {
        Self {
            _args: PhantomData,
            name: None,
            description: None,
            execute: None,
        }
    }
}

impl<Func, Args, Fut> From<Func> for FunctionBuilder<Func, Args, Fut>
where
    Func: FnMut(Args) -> Fut + Send + 'static,
    Args: FunctionArguments + DeserializeOwned + Send,
    Fut: Future<Output = anyhow::Result<String>> + Send,
{
    fn from(value: Func) -> Self {
        Self {
            name: Some(
                std::any::type_name::<Func>()
                    .split("::")
                    .last()
                    .unwrap()
                    .into(),
            ),
            description: None,
            _args: PhantomData,
            execute: Some(Arc::new(Mutex::new(value))),
        }
    }
}

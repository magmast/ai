use std::{marker::PhantomData, sync::Arc};

use futures::{
    future::{self, BoxFuture},
    Future, FutureExt, TryFutureExt,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

pub struct FunctionDeclaration {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) args: Value,
}

impl From<&Function> for FunctionDeclaration {
    fn from(func: &Function) -> Self {
        Self {
            name: func.name.clone(),
            description: func.description.clone(),
            args: func.args.clone(),
        }
    }
}

pub(crate) type BoxExecute = Box<dyn FnMut(&str) -> BoxFuture<'static, String>>;

pub struct Function {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) args: Value,
    pub(crate) execute: BoxExecute,
}

impl From<BoxFunctionBuilder> for Function {
    fn from(fb: BoxFunctionBuilder) -> Self {
        Self {
            name: fb.name,
            description: fb.description,
            args: fb.args,
            execute: fb.execute,
        }
    }
}

impl<Marker> From<MarkedFunctionBuilder<Marker>> for Function {
    fn from(mfb: MarkedFunctionBuilder<Marker>) -> Self {
        BoxFunctionBuilder::from(mfb).into()
    }
}

pub trait FunctionBuilder<T> {
    fn description(self, description: impl ToString) -> T;
}

pub struct BoxFunctionBuilder {
    name: String,
    description: Option<String>,
    args: Value,
    execute: BoxExecute,
}

impl<Marker> From<MarkedFunctionBuilder<Marker>> for BoxFunctionBuilder {
    fn from(mfb: MarkedFunctionBuilder<Marker>) -> Self {
        Self {
            name: mfb.name,
            description: mfb.description,
            args: mfb.args,
            execute: mfb.execute,
        }
    }
}

impl FunctionBuilder<Self> for BoxFunctionBuilder {
    fn description(self, description: impl ToString) -> Self {
        Self {
            description: Some(description.to_string()),
            ..self
        }
    }
}

pub struct MarkedFunctionBuilder<Marker> {
    _marker: PhantomData<Marker>,
    name: String,
    description: Option<String>,
    args: Value,
    execute: BoxExecute,
}

impl<Func, Args, Fut, Out> From<Func> for MarkedFunctionBuilder<(Args, Fut, Out)>
where
    Func: Fn(Args) -> Fut + Send + Sync + 'static,
    Args: FunctionArguments + DeserializeOwned + Send + 'static,
    Fut: Future<Output = Out> + Send + 'static,
    Out: Serialize,
{
    fn from(func: Func) -> Self {
        let func = Arc::new(func);

        Self {
            _marker: PhantomData,
            name: std::any::type_name::<Func>()
                .split("::")
                .last()
                .unwrap()
                .into(),
            description: None,
            args: Args::json_schema(),
            execute: Box::new(move |args| {
                let func = Arc::clone(&func);
                let args = serde_json::from_str(args);

                future::ready(args)
                    .map_ok(move |args| (func)(args))
                    .and_then(|output| async move { serde_json::to_string(&output.await) })
                    .map(|result| match result {
                        Ok(s) => s,
                        Err(err) => err.to_string(),
                    })
                    .boxed()
            }),
        }
    }
}

impl<T, Marker> FunctionBuilder<MarkedFunctionBuilder<Marker>> for T
where
    T: Into<MarkedFunctionBuilder<Marker>>,
{
    fn description(self, description: impl ToString) -> MarkedFunctionBuilder<Marker> {
        MarkedFunctionBuilder {
            description: Some(description.to_string()),
            ..self.into()
        }
    }
}

pub trait FunctionArguments {
    fn json_schema() -> Value;
}

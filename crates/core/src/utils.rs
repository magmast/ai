use async_trait::async_trait;
use futures::Future;

#[async_trait]
pub trait FlattenFutureResultExt<T, E> {
    async fn flatten_future(self) -> Result<T, E>;
}

#[async_trait]
impl<T, E, F> FlattenFutureResultExt<T, E> for Result<F, E>
where
    F: Future<Output = Result<T, E>> + Send,
    E: Send,
{
    async fn flatten_future(self) -> Result<T, E> {
        match self {
            Ok(future) => future.await,
            Err(err) => Err(err),
        }
    }
}

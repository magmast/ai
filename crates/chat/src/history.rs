use std::{convert::Infallible, path::PathBuf};

use async_trait::async_trait;

use crate::dto::Message;

#[async_trait]
pub trait History {
    type Err;

    async fn read(&mut self) -> Result<Vec<Message>, Self::Err>;
    async fn write(
        &mut self,
        messages: impl AsRef<[Message]> + Send + Sync,
    ) -> Result<(), Self::Err>;
}

#[derive(Default)]
pub struct NoopHistory;

#[async_trait]
impl History for NoopHistory {
    type Err = Infallible;

    async fn read(&mut self) -> Result<Vec<Message>, Self::Err> {
        Ok(vec![])
    }

    async fn write(
        &mut self,
        _messages: impl AsRef<[Message]> + Send + Sync,
    ) -> Result<(), Self::Err> {
        Ok(())
    }
}

#[cfg(test)]
mod noop_history_tests {
    use crate::{
        dto::Message,
        history::{History, NoopHistory},
    };

    #[tokio::test]
    async fn should_read_empty_vec_always() {
        let mut h = NoopHistory;

        assert_eq!(Ok(vec![]), h.read().await, "Expected empty vec.");

        h.write([Message::System("".to_string())]).await.unwrap();
        assert_eq!(Ok(vec![]), h.read().await, "Expected empty vec.");

        for _ in 0..100 {
            h.write([Message::User("".to_string())]).await.unwrap();
        }
        assert_eq!(Ok(vec![]), h.read().await, "Expected empty vec.");
    }
}

pub struct SocketHistory<T: Socket>(T);

impl<T: Socket> SocketHistory<T> {
    pub fn new(socket: T) -> Self {
        SocketHistory(socket)
    }
}

impl SocketHistory<FileSocket> {
    pub fn file(path: impl Into<PathBuf>) -> Self {
        Self::new(FileSocket(path.into()))
    }
}

#[async_trait]
pub trait Socket {
    async fn read(&mut self) -> std::io::Result<Vec<u8>>;
    async fn write(&mut self, bytes: impl AsRef<[u8]> + Send) -> std::io::Result<()>;
}

pub struct FileHistory(SocketHistory<FileSocket>);

impl FileHistory {
    pub fn new(path: impl Into<PathBuf>) -> FileHistory {
        Self(SocketHistory::file(path))
    }
}

#[async_trait]
impl History for FileHistory {
    type Err = std::io::Error;

    async fn read(&mut self) -> Result<Vec<Message>, Self::Err> {
        self.0
             .0
            .read()
            .await
            .and_then(|bytes| Ok(serde_json::from_slice(&bytes)?))
            .or_else(|err| {
                if err.kind() == std::io::ErrorKind::NotFound {
                    Ok(vec![])
                } else {
                    Err(err)
                }
            })
    }

    async fn write(
        &mut self,
        messages: impl AsRef<[Message]> + Send + Sync,
    ) -> Result<(), Self::Err> {
        if let Some(parent) = self.0 .0 .0.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let bytes = serde_json::to_vec(messages.as_ref())?;
        self.0 .0.write(&bytes).await?;
        Ok(())
    }
}

struct FileSocket(PathBuf);

#[async_trait]
impl Socket for FileSocket {
    async fn read(&mut self) -> std::io::Result<Vec<u8>> {
        tokio::fs::read(&self.0).await
    }

    async fn write(&mut self, bytes: impl AsRef<[u8]> + Send) -> std::io::Result<()> {
        tokio::fs::write(&self.0, bytes).await
    }
}

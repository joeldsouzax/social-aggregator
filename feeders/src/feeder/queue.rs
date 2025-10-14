use crate::error::Error;
use std::fmt::Debug;
use tokio::sync::mpsc::Sender;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct FeederQueue<T>
where
    T: Debug,
{
    tx: Sender<T>,
}

impl<T> FeederQueue<T>
where
    T: Debug,
{
    pub fn create(tx: Sender<T>) -> Self {
        Self { tx }
    }

    #[instrument(level = "info", err)]
    pub async fn send(&self, message: T) -> Result<(), Error> {
        self.tx
            .send(message)
            .await
            .map_err(|err| Error::FeederSend(err.to_string()))
    }
}

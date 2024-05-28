use crate::{Connection, Shutdown};

use tokio::sync::mpsc;
use tracing::debug;

#[derive(Debug)]
pub(crate) struct Handler {
    //nats:
    pub(crate) connection: Connection,
    pub(crate) shutdown: Shutdown,
    pub(crate) _shutdown_complete: mpsc::Sender<()>,
}

impl Handler {
    pub(crate) async fn run(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        while !self.shutdown.is_shutdown() {
            let maybe_data = tokio::select! {
                res = self.connection.read_data() => res?,
                _ = self.shutdown.recv() => {
                    return Ok(());
                }
            };

            let data = match maybe_data {
                Some(data) => data,
                None => return Ok(()),
            };

            debug!("{:?}", data);
        }
        Ok(())
    }
}

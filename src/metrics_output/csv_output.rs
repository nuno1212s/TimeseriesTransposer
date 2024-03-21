use csv::Writer;
use futures::{Stream, StreamExt};
use serde::Serialize;
use std::io::Write;
use tokio::task::JoinError;

use crate::metrics_output::MetricOutput;

pub struct CSVCreator<W> {
    pub writer: W,
}

impl<M, W> MetricOutput<M> for CSVCreator<W>
where
    W: Write + Send + 'static,
{
    type Error = JoinError;

    async fn export_metrics(self, metrics: impl Stream<Item = M>) -> Result<(), Self::Error>
    where
        M: Serialize + Send + 'static,
    {
        let (c_tx, c_rx) = flume::bounded(128);

        tokio::task::spawn_blocking(move || {
            let mut writer = Writer::from_writer(self.writer);

            while let Ok(metric) = c_rx.recv() {
                writer.serialize(metric).expect("Failed to serialize");
            }
        });

        metrics
            .for_each(|metric| async {
                c_tx.send_async(metric).await.expect("Failed to send");
            })
            .await;

        Ok(())
    }
}

pub mod csv_output;

use futures::Stream;
use serde::Serialize;
use std::error::Error;

/// The output trait for outputting metrics
pub trait MetricOutput<M> {
    type Error: Error + Sync + Send;

    async fn export_metrics(self, metrics: impl Stream<Item = M>) -> Result<(), Self::Error>
    where
        M: Serialize + Send + 'static;
}

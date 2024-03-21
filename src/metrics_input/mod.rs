pub mod influx_input;

use futures::stream::BoxStream;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// The metrics input trait, defined to
pub trait MetricsInput<M>
where
    M: for<'a> Deserialize<'a>,
{
    type Error: Error + Sync + Send;

    /// Load metrics for a given argument
    async fn load_metrics_for_args(
        &self,
        zone: impl Into<String>,
        tag_args: &[(&str, &str)],
    ) -> Result<BoxStream<M>, Self::Error>;
}

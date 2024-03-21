use chrono::{DateTime, Utc};
use futures::stream::BoxStream;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use crate::metrics_input::influx_input::{InfluxArgs, InfluxInput};
use crate::metrics_input::MetricsInput;
use crate::metrics_output::csv_output::CSVCreator;
use crate::metrics_output::MetricOutput;

mod metrics_input;
mod metrics_output;

#[derive(Serialize, Deserialize, Debug)]
pub struct Metric {
    time: DateTime<Utc>,
    extra: String,
    host: String,
    value: f64,
}

#[tokio::main]
async fn main() {
    let username = std::env::var("INFLUX_USERNAME").ok();
    let password = std::env::var("INFLUX_PASSWORD").ok();

    let input = InfluxArgs {
        url: std::env::var("INFLUX_URL").expect("INFLUX_URL not set"),
        database: std::env::var("INFLUX_DB").expect("INFLUX_DB not set"),
        auth: username.and_then(|usr| password.map(|pwd| (usr, pwd))),
    };

    let influx_input = InfluxInput::new(input);

    let metric_stream: BoxStream<Metric> = influx_input
        .load_metrics_for_args("OPS_PER_SECOND", &[("host", "NodeId(2)")])
        .await
        .expect("Failed to load metrics");

    let csv_output = CSVCreator {
        writer: std::io::stdout(),
    };

    csv_output
        .export_metrics(metric_stream)
        .await
        .expect("Failed to export metrics");
}

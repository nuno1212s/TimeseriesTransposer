use crate::metrics_input::MetricsInput;
use futures::stream::BoxStream;
use futures::{Stream, StreamExt};
use influxdb::{Error, Query, ReadQuery};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct InfluxArgs<URL, DB, USR, PWD>
where
    URL: Into<String>,
    DB: Into<String>,
    USR: Into<String>,
    PWD: Into<String>,
{
    pub url: URL,
    pub database: DB,
    pub auth: Option<(USR, PWD)>,
}

pub struct InfluxInput {
    client: influxdb::Client,
}

impl InfluxInput {
    pub fn new<URL, DB, USR, PWD>(args: InfluxArgs<URL, DB, USR, PWD>) -> Self
    where
        URL: Into<String>,
        DB: Into<String>,
        USR: Into<String>,
        PWD: Into<String>,
    {
        let InfluxArgs {
            url,
            database,
            auth,
        } = args;

        let mut client = influxdb::Client::new(url, database);

        if let Some((usr, pwd)) = auth {
            client = client.with_auth(usr, pwd);
        }

        Self { client }
    }
}

impl<M> MetricsInput<M> for InfluxInput
where
    M: for<'a> Deserialize<'a> + Send + 'static,
{
    type Error = Error;

    async fn load_metrics_for_args(
        &self,
        series: impl Into<String>,
        tag_args: &[(&str, &str)],
    ) -> Result<BoxStream<M>, Self::Error> {
        let where_clauses = tag_args
            .iter()
            .map(|(tag_name, tag_value)| format!("{}='{}'", tag_name, tag_value))
            .collect::<Vec<String>>()
            .join(" AND ");

        let read_query = ReadQuery::new(format!(
            "SELECT * FROM {} WHERE {}",
            series.into(),
            where_clauses
        ));

        let read_result = self.client.query(read_query).await?;

        let (c_tx, c_rx) = flume::bounded(128);

        tokio::task::spawn_blocking(move || {
            let json: Value = serde_json::from_str(&read_result).expect("Failed to parse JSON");

            json["results"]
                .as_array()
                .expect("How is this not an array?")
                .iter()
                .for_each(|statement| {
                    statement["series"]
                        .as_array()
                        .expect("How are series not an array")
                        .iter()
                        .for_each(|series| {
                            let values = series["values"]
                                .as_array()
                                .expect("How are values not an array");

                            values.iter().for_each(|value| {
                                let metric = serde_json::from_value(value.clone())
                                    .expect("Failed to parse value");

                                c_tx.send(metric).expect("Failed to send");
                            });
                        });
                });
        });

        Ok(c_rx.into_stream().boxed())
    }
}

/// *NOTE must create "bucket" in influxdbd
use futures::prelude::*;
use influxdb2::models::DataPoint;
use influxdb2::Client;
struct InfluxDb {
    client: Client,
    bucket: String,
}

impl InfluxDb {
    pub fn new(bucket: String) -> Self {
        let host = std::env::var("INFLUXDB_HOST").unwrap();
        let org = std::env::var("INFLUXDB_ORG").unwrap();
        let token = std::env::var("INFLUXDB_TOKEN").unwrap();
        let client = Client::new(host, org, token);
        Self { client, bucket }
    }
    async fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        let points = vec![
            DataPoint::builder("cpu")
                .tag("host", "server01")
                .field("usage", 0.5)
                .build()?,
            DataPoint::builder("cpu")
                .tag("host", "server01")
                .tag("region", "us-west")
                .field("usage", 0.87)
                .build()?,
        ];
        let result = self
            .client
            .write(&self.bucket[..], stream::iter(points))
            .await?;
        Ok(())
    }
}
async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let host = std::env::var("INFLUXDB_HOST").unwrap();
    let org = std::env::var("INFLUXDB_ORG").unwrap();
    let token = std::env::var("INFLUXDB_TOKEN").unwrap();
    let bucket = "bucket"; // *NOTE must create bucket in influxdb
    let client = Client::new(host, org, token);
    dbg!(&client);

    let points = vec![
        DataPoint::builder("cpu")
            .tag("host", "server01")
            .field("usage", 0.5)
            .build()?,
        DataPoint::builder("cpu")
            .tag("host", "server01")
            .tag("region", "us-west")
            .field("usage", 0.87)
            .build()?,
    ];

    client.write(bucket, stream::iter(points)).await?;

    Ok(())
}

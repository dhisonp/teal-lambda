use aws_sdk_dynamodb::{types::AttributeValue, Client, Error};
use serde_dynamo::to_item;

pub struct DynamoClient {
    client: Client,
}

const TABLE_NAME: &str = "teal-db";

impl DynamoClient {
    pub async fn init() -> Self {
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);
        Self { client }
    }

    pub async fn ping(&self) -> Result<bool, Error> {
        self.client
            .describe_table()
            .table_name(TABLE_NAME)
            .send()
            .await?;

        Ok(true)
    }

    pub async fn put(&self, key: &str, item: serde_json::Value) -> anyhow::Result<bool> {
        let _item = to_item(item)?;
        let av = AttributeValue::M(_item);
        let req = self.client.put_item().table_name(TABLE_NAME).item(key, av);

        req.send().await?;
        Ok(true)
    }
}

// Global database client instance (optional pattern for Lambda)
// pub type SharedDatabaseClient = Arc<DynamoClient>;

use aws_sdk_dynamodb::{
    operation::create_table::CreateTableOutput,
    types::{
        AttributeDefinition, AttributeValue, BillingMode, KeySchemaElement, KeyType,
        ScalarAttributeType,
    },
    Client, Error,
};
use serde_dynamo::to_item;

pub struct DynamoClient {
    client: Client,
}

const TABLE_NAME: &str = "teal-db";
const KEY: &str = "tealant_id";

impl DynamoClient {
    pub async fn init() -> Self {
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);
        Self { client }
    }

    async fn create_table(&self) -> anyhow::Result<CreateTableOutput> {
        let ad = AttributeDefinition::builder()
            .attribute_name(KEY)
            .attribute_type(ScalarAttributeType::S)
            .build()?;

        let ks = KeySchemaElement::builder()
            .attribute_name(KEY)
            .key_type(KeyType::Hash)
            .build()?;

        let res = self
            .client
            .create_table()
            .table_name(TABLE_NAME)
            .key_schema(ks)
            .attribute_definitions(ad)
            .billing_mode(BillingMode::PayPerRequest)
            .send()
            .await?; // This will automatically convert the error to anyhow::Error

        println!("Added table {} with key {}", TABLE_NAME, KEY);
        Ok(res)
    }

    async fn check_table_exists(&self) -> ::anyhow::Result<bool> {
        let paginator = self.client.list_tables().into_paginator().items().send();
        let table_names = paginator.collect::<Result<Vec<_>, _>>().await?;
        Ok(table_names.iter().any(|name| name == TABLE_NAME))
    }

    /// Check if table exists and creates if it doesn't.
    pub async fn check_create_table(&self) -> anyhow::Result<bool> {
        let exists = self.check_table_exists().await?;
        if exists {
            return Ok(false);
        }

        self.create_table().await?;
        Ok(true)
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

use aws_sdk_dynamodb::{
    operation::create_table::CreateTableOutput,
    types::{AttributeDefinition, BillingMode, KeySchemaElement, KeyType, ScalarAttributeType},
    Client,
};
use serde_dynamo::to_item;
use std::sync::{Arc, OnceLock};

pub struct DynamoClient {
    client: Client,
}

pub const USERS_TABLE_NAME: &str = "teal-users";
pub const TELLS_TABLE_NAME: &str = "teal-tells";
pub const KEY: &str = "tid";

static DB_CLIENT: OnceLock<Arc<DynamoClient>> = OnceLock::new();

pub fn init_global_db(client: DynamoClient) {
    DB_CLIENT.set(Arc::new(client)).ok();
}

pub fn use_db() -> &'static Arc<DynamoClient> {
    DB_CLIENT.get().expect("Database not initialized")
}

impl DynamoClient {
    pub async fn init() -> Self {
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);
        Self { client }
    }

    async fn create_table(&self, table_name: &str) -> anyhow::Result<CreateTableOutput> {
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
            .table_name(table_name)
            .key_schema(ks)
            .attribute_definitions(ad)
            .billing_mode(BillingMode::PayPerRequest)
            .send()
            .await?; // This will automatically convert the error to anyhow::Error

        println!("Added table {} with key {}", table_name, KEY);
        Ok(res)
    }

    async fn check_table_exists(&self, table_name: &str) -> ::anyhow::Result<bool> {
        let paginator = self.client.list_tables().into_paginator().items().send();
        let table_names = paginator.collect::<Result<Vec<_>, _>>().await?;
        Ok(table_names.iter().any(|name| name == table_name))
    }

    /// Check if table exists and creates if it doesn't.
    pub async fn check_create_table(&self, table_name: &str) -> anyhow::Result<bool> {
        let exists = self.check_table_exists(table_name).await?;
        if exists {
            return Ok(false);
        }

        self.create_table(table_name).await?;
        Ok(true)
    }

    pub async fn ping(&self) -> Result<bool, aws_sdk_dynamodb::Error> {
        self.client
            .describe_table()
            .table_name(USERS_TABLE_NAME)
            .send()
            .await?;

        Ok(true)
    }

    pub async fn put(&self, table_name: &str, item: serde_json::Value) -> anyhow::Result<bool> {
        let _item = to_item(item)?;
        let req = self
            .client
            .put_item()
            .table_name(table_name)
            .set_item(Some(_item));

        req.send().await?;
        Ok(true)
    }
}

pub async fn initialize_db() -> anyhow::Result<bool> {
    let db = DynamoClient::init().await;

    // TODO: There should be a better way instead of manually
    //   calling them one by one.
    db.check_create_table(USERS_TABLE_NAME).await?;
    db.check_create_table(TELLS_TABLE_NAME).await?;

    match db.ping().await {
        Ok(_) => println!("Successfully connected to DynamoDB!"),
        Err(e) => {
            eprintln!("DynamoDB failed connection: {:?}", e);
            return Err(e.into());
        }
    }

    init_global_db(db); // Store the DynamoDB client in a global OnceLock
    return Ok(true);
}

// Global database client instance (optional pattern for Lambda)
// pub type SharedDatabaseClient = Arc<DynamoClient>;

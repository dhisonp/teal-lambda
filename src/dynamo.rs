use aws_sdk_dynamodb as dynamodb;
use std::sync::Arc;

pub struct DynamoClient {
    client: dynamodb::Client,
}

impl DynamoClient {
    pub async fn init() -> Self {
        let config = aws_config::load_from_env().await;
        let client = dynamodb::Client::new(&config);
        Self { client }
    }

    pub fn from_client(client: dynamodb::Client) -> Self {
        Self { client }
    }

    pub async fn list_tables(&self) -> Result<Vec<String>, dynamodb::Error> {
        let paginator = self.client.list_tables().into_paginator().items().send();
        let table_names = paginator.collect::<Result<Vec<_>, _>>().await?;

        println!("Tables:");
        for name in &table_names {
            println!("  {}", name);
        }
        println!("Found {} tables", table_names.len());

        Ok(table_names)
    }
}

/// Global database client instance (optional pattern for Lambda)
pub type SharedDatabaseClient = Arc<DynamoClient>;

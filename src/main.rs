use lambda_http::{run, service_fn, tracing, Error};
mod dynamo;
mod gemini;
mod http_handler;
mod schema;
mod users;

use http_handler::function_handler;

use crate::dynamo::USERS_TABLE_NAME;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    // TODO: Do not load .env in production
    dotenvy::dotenv()?;

    // TODO: Refactor into db crate
    let db = dynamo::DynamoClient::init().await;
    db.check_create_table(USERS_TABLE_NAME).await?;
    match db.ping().await {
        Ok(_) => println!("DynamoDB connected"),
        Err(e) => {
            eprintln!("DynamoDB failed connection: {:?}", e);
            return Err(Error::from(e));
        }
    }
    dynamo::init_global_db(db); // Store the DynamoDB client in a global OnceLock

    run(service_fn(function_handler)).await
}

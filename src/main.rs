mod dynamo;
mod gemini;
mod http_handler;
mod prompts;
mod schema;
mod users;

use crate::dynamo::initialize_db;
use http_handler::function_handler;
use lambda_http::{run, service_fn, tracing, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    dotenvy::dotenv()?; // TODO: Do not load .env in production
    initialize_db().await?;
    run(service_fn(function_handler)).await
}

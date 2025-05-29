use lambda_http::{run, service_fn, tracing, Error};
mod http_handler;
mod gemini;

use http_handler::function_handler;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    // TODO: Do not load .env in production
    dotenvy::dotenv()?;

    run(service_fn(function_handler)).await
}

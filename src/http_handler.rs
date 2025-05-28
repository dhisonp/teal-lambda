use crate::openai_client::ask;
use lambda_http::{Body, Error, Request, RequestExt, Response};
use serde::Serialize;

#[derive(Serialize)]
struct Reply {
    body: String,
}

pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let username = event
        .query_string_parameters_ref()
        .and_then(|params| params.first("name"))
        .unwrap_or("stranger");

    let prompt =
        format!("Please say a very warm welcome and hello to me, where my name is {username}");
    let response = ask(&prompt).await.to_string();
    let data = Reply {
        body: response.clone(),
    };

    // Can we improve error handling?
    let resp = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(serde_json::to_string(&data)?.into())
        .map_err(Box::new)?; // What does this do?
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_http::{Request, RequestExt};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_generic_http_handler() {
        let request = Request::default();

        let response = function_handler(request).await.unwrap();
        assert_eq!(response.status(), 200);

        let body_bytes = response.body().to_vec();
        let body_string = String::from_utf8(body_bytes).unwrap();

        assert_eq!(
            body_string,
            "Hello world, this is an AWS Lambda HTTP request"
        );
    }

    #[tokio::test]
    async fn test_http_handler_with_query_string() {
        let mut query_string_parameters: HashMap<String, String> = HashMap::new();
        query_string_parameters.insert("name".into(), "teal-lambda".into());

        let request = Request::default().with_query_string_parameters(query_string_parameters);

        let response = function_handler(request).await.unwrap();
        assert_eq!(response.status(), 200);

        let body_bytes = response.body().to_vec();
        let body_string = String::from_utf8(body_bytes).unwrap();

        assert_eq!(
            body_string,
            "Hello teal-lambda, this is an AWS Lambda HTTP request"
        );
    }
}

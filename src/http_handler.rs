use crate::gemini;
use lambda_http::{Body, Error, Request, RequestExt, Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ResponseBody {
    tell: Option<String>,
    error_message: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct RequestBody {
    text: String,
}

fn parse_request(event: &Request) -> Result<(String, RequestBody), String> {
    if event.body().is_empty() {
        return Err("Request body required".to_string());
    }

    let body: RequestBody =
        serde_json::from_slice(event.body()).map_err(|_| "Invalid JSON body")?;
    if body.text.trim().is_empty() {
        return Err("text cannot be an empty string".to_string());
    }

    let username = event
        .query_string_parameters_ref()
        .and_then(|p| p.first("name"))
        .ok_or("name parameter is required")?
        .to_string();

    Ok((username, body))
}

pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let (username, body) = match parse_request(&event) {
        Ok(data) => data,
        Err(msg) => {
            let data = ResponseBody {
                tell: None,
                error_message: Some(msg),
            };
            return Ok(Response::builder()
                .status(422)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&data)?.into())
                .map_err(Box::new)?);
        }
    };

    let answer = gemini::tell(&username, &body.text, None).await?;
    let data = ResponseBody {
        tell: Some(answer),
        error_message: None,
    };

    let res = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(serde_json::to_string(&data)?.into())
        .map_err(Box::new)?;

    Ok(res)
}

// TODO: Do not forget to update tests upon MVP
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

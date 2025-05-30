use crate::users::create_user;
use crate::{gemini, users::User};
use lambda_http::{http, Body, Error, Request, RequestExt, Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct RequestTellBody {
    text: String,
}

#[derive(Serialize)]
struct ResponseTellBody {
    tell: Option<String>,
    summary: Option<String>,
    state: Option<String>,
    error_message: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct RequestPostUserCreate {
    name: String,
}

// #[derive(Serialize)]
// struct ResponsePostUserCreate {
//     message: String,
// }

pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let method = event.method();
    let path = event.uri().path();

    match (method, path) {
        (&http::Method::POST, "/tell") => post_tell(event).await,
        (&http::Method::POST, "/user/create") => post_user_create(event).await,
        _ => {
            let data = ResponseTellBody {
                tell: None,
                error_message: Some("Route does not exist".to_string()),
                state: None,
                summary: None,
            };
            return Ok(Response::builder()
                .status(http::StatusCode::NOT_FOUND)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&data)?.into())
                .map_err(Box::new)?);
        }
    }
}

async fn post_tell(event: Request) -> Result<Response<Body>, Error> {
    fn parse_request(event: &Request) -> Result<(String, RequestTellBody), String> {
        if event.body().is_empty() {
            return Err("Request body required".to_string());
        }

        let body: RequestTellBody =
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

    let (username, body) = match parse_request(&event) {
        Ok(data) => data,
        Err(msg) => {
            // Is there a way to not include None keys?
            let data = ResponseTellBody {
                tell: None,
                error_message: Some(msg),
                state: None,
                summary: None,
            };
            return Ok(Response::builder()
                .status(http::StatusCode::UNPROCESSABLE_ENTITY)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&data)?.into())
                .map_err(Box::new)?);
        }
    };

    // TODO: Delegate business logic elsewhere, not the controller
    let answer = gemini::tell(&username, &body.text, None).await?;
    let summary = gemini::summarize_tell(&answer).await?;
    let state = gemini::generate_state(None, Some(&summary)).await?;
    let data = ResponseTellBody {
        tell: Some(answer),
        summary: Some(summary),
        state: Some(state),
        error_message: None,
    };

    let res = Response::builder()
        .status(http::StatusCode::OK)
        .header("content-type", "application/json")
        .body(serde_json::to_string(&data)?.into())
        .map_err(Box::new)?;

    Ok(res)
}

/// Creates a new user.
/// TODO: Handle errors gracefully.
async fn post_user_create(event: Request) -> Result<Response<Body>, Error> {
    // TODO: Determine what should be here!
    // let path_param = event.path_parameters_ref();
    // if path_param.map_or(true, |p| p.is_empty()) {
    //     return Err("Invalid path parameter".into());
    // }
    let body: RequestPostUserCreate =
        serde_json::from_slice(event.body()).map_err(|_| "Invalid JSON body")?;

    let data = User {
        tealant_id: uuid::Uuid::new_v4().to_string(),
        name: body.name,
        created_at: chrono::Utc::now().to_rfc3339(),
        current_mood: None,
        current_state: None,
        summary_history: None,
        tell_history: None,
    };

    create_user(&data).await?;

    // TODO: Correct response format
    let res = Response::builder()
        .status(http::StatusCode::CREATED)
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

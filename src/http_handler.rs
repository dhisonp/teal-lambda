use crate::gemini::tell;
use crate::schema::User;
use crate::users::create_user;
use lambda_http::{http, Body, Error, Request, RequestExt, Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ResponseBody {
    success: bool,
    error_message: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct RequestBodyTell {
    text: String,
}

#[derive(Serialize)]
struct ResponseBodyTell {
    base: ResponseBody,
    tell: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct RequestBodyPostUserCreate {
    name: String,
    email: String,
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
            let data = ResponseBody {
                success: false,
                error_message: Some("Route not found".to_string()),
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
    fn parse_request(event: &Request) -> Result<(String, RequestBodyTell), String> {
        if event.body().is_empty() {
            return Err("Request body required".to_string());
        }

        let body: RequestBodyTell =
            serde_json::from_slice(event.body()).map_err(|_| "Invalid JSON body")?;
        if body.text.trim().is_empty() {
            return Err("text cannot be an empty string".to_string());
        }

        let username = event
            .query_string_parameters_ref()
            .and_then(|p| p.first("username"))
            .ok_or("missing username query param")?
            .to_string();

        Ok((username, body))
    }

    let (username, body) = match parse_request(&event) {
        Ok(data) => data,
        Err(msg) => {
            // Is there a way to not include None keys?
            let data = ResponseBody {
                success: false,
                error_message: Some(msg),
            };
            return Ok(Response::builder()
                .status(http::StatusCode::UNPROCESSABLE_ENTITY)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&data)?.into())
                .map_err(Box::new)?);
        }
    };

    // NOTE: This is commented due to bad logging/error display. Find a way to
    //   better log errors and uncomment to allow for better client experience.
    // let answer = match tell(&username, &body.text, None).await {
    //     Ok(data) => data,
    //     Err(_) => {
    //         // TODO: Refactor duplicate return error logic
    //         let data = ResponseBody {
    //             success: false,
    //             error_message: Some("Oops! An error occurred when telling your story.".to_string()),
    //         };
    //         return Ok(Response::builder()
    //             .status(http::StatusCode::UNPROCESSABLE_ENTITY)
    //             .header("content-type", "application/json")
    //             .body(serde_json::to_string(&data)?.into())
    //             .map_err(Box::new)?);
    //     }
    // };
    let answer = tell(&username, &body.text, None).await?;

    let data = ResponseBodyTell {
        base: ResponseBody {
            success: true,
            error_message: None,
        },
        tell: Some(answer),
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
    let data: RequestBodyPostUserCreate =
        serde_json::from_slice(event.body()).map_err(|_| "Invalid JSON body")?;

    let data = User {
        tid: uuid::Uuid::new_v4().to_string(),
        name: data.name,
        email: data.email,
        created_at: chrono::Utc::now().to_rfc3339(),
        current_mood: None,
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

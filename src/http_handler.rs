use crate::tell::tell;
use crate::schema::User;
use crate::users::create_user;
use lambda_http::{http, Body, Error, Request, RequestExt, Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
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

// TODO: Validate if user exists
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

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_http::{http::Method, Body, Request};

    #[tokio::test]
    async fn test_route_not_found() {
        let request = Request::default();
        let response = function_handler(request).await.unwrap();
        
        assert_eq!(response.status(), 404);
        let body: ResponseBody = serde_json::from_slice(response.body()).unwrap();
        assert!(!body.success);
        assert_eq!(body.error_message, Some("Route not found".to_string()));
    }

    fn create_test_request(method: Method, path: &str, body: Body) -> Request {
        let mut req = Request::new(body);
        *req.method_mut() = method;
        *req.uri_mut() = path.parse().unwrap();
        req
    }

    #[tokio::test]
    async fn test_post_tell_missing_body() {
        let request = create_test_request(Method::POST, "/tell?username=testuser", Body::Empty);

        let response = function_handler(request).await.unwrap();
        assert_eq!(response.status(), 422);
        
        let body: ResponseBody = serde_json::from_slice(response.body()).unwrap();
        assert!(!body.success);
        assert_eq!(body.error_message, Some("Request body required".to_string()));
    }

    #[tokio::test]
    async fn test_post_tell_invalid_json() {
        let request = create_test_request(
            Method::POST, 
            "/tell?username=testuser", 
            Body::Text("invalid json".to_string())
        );

        let response = function_handler(request).await.unwrap();
        assert_eq!(response.status(), 422);
        
        let body: ResponseBody = serde_json::from_slice(response.body()).unwrap();
        assert!(!body.success);
        assert_eq!(body.error_message, Some("Invalid JSON body".to_string()));
    }

    #[tokio::test]
    async fn test_post_tell_empty_text() {
        let request_body = RequestBodyTell {
            text: "".to_string(),
        };
        let json = serde_json::to_string(&request_body).unwrap();
        
        let request = create_test_request(
            Method::POST,
            "/tell?username=testuser",
            Body::Text(json)
        );

        let response = function_handler(request).await.unwrap();
        assert_eq!(response.status(), 422);
        
        let body: ResponseBody = serde_json::from_slice(response.body()).unwrap();
        assert!(!body.success);
        assert_eq!(body.error_message, Some("text cannot be an empty string".to_string()));
    }

    #[tokio::test]
    async fn test_post_tell_missing_username() {
        let request_body = RequestBodyTell {
            text: "I'm feeling great!".to_string(),
        };
        let json = serde_json::to_string(&request_body).unwrap();
        
        let request = create_test_request(Method::POST, "/tell", Body::Text(json));

        let response = function_handler(request).await.unwrap();
        assert_eq!(response.status(), 422);
        
        let body: ResponseBody = serde_json::from_slice(response.body()).unwrap();
        assert!(!body.success);
        assert_eq!(body.error_message, Some("missing username query param".to_string()));
    }

    #[test]
    fn test_request_body_tell_serialization() {
        let body = RequestBodyTell {
            text: "Hello world".to_string(),
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("Hello world"));
        
        let parsed: RequestBodyTell = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.text, "Hello world");
    }

    #[test]
    fn test_request_body_user_create_serialization() {
        let body = RequestBodyPostUserCreate {
            name: "Jane Doe".to_string(),
            email: "jane@example.com".to_string(),
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("Jane Doe"));
        assert!(json.contains("jane@example.com"));
        
        let parsed: RequestBodyPostUserCreate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Jane Doe");
        assert_eq!(parsed.email, "jane@example.com");
    }

    #[test]
    fn test_response_body_serialization() {
        let body = ResponseBody {
            success: true,
            error_message: None,
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("true"));
        
        let body_with_error = ResponseBody {
            success: false,
            error_message: Some("Error occurred".to_string()),
        };
        let json = serde_json::to_string(&body_with_error).unwrap();
        assert!(json.contains("false"));
        assert!(json.contains("Error occurred"));
    }
}

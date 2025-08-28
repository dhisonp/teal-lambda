use serde::Serialize;
use serde_json::to_value;

use crate::dynamo::{use_db, USERS_TABLE_NAME};

// TODO: Storing OAuth2.0 credentials
#[derive(Serialize)]
pub struct User {
    pub tid: String,
    pub name: String,
    pub email: String,
    pub current_mood: Option<String>,
    pub created_at: String, // TODO: Use chrono::DateTime<Utc>
}

pub async fn create_user(data: &User) -> anyhow::Result<bool> {
    let db = use_db();
    db.put(USERS_TABLE_NAME, to_value(data)?).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_serialization() {
        let user = User {
            tid: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            current_mood: Some("happy".to_string()),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("John Doe"));
        assert!(json.contains("john@example.com"));
        assert!(json.contains("happy"));
    }

    #[test]
    fn test_user_with_none_mood() {
        let user = User {
            tid: "123".to_string(),
            name: "Jane".to_string(),
            email: "jane@test.com".to_string(),
            current_mood: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("Jane"));
        assert!(json.contains("null") || !json.contains("current_mood"));
    }
}

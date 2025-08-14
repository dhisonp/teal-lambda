use serde::Serialize;

use crate::gemini::TellItem;

// TODO: Storing OAuth2.0 credentials
#[derive(Serialize)]
pub struct User {
    pub tid: String,
    pub name: String,
    pub email: String,
    pub current_mood: Option<String>,
    pub created_at: String, // TODO: Use chrono::DateTime<Utc>
}

// TODO: A better structure for better LLM understanding
pub struct Context {
    pub tells: Vec<TellItem>,
}

use crate::dynamo;
use crate::schema::Mood;
use serde::Serialize;

#[derive(Serialize)]
pub struct User {
    pub tealant_id: String,
    pub name: String,
    pub created_at: String,
    pub summary_history: Option<Vec<String>>,
    pub tell_history: Option<Vec<String>>,
    pub current_mood: Option<Mood>,
    pub current_state: Option<String>,
}

const TABLE_NAME: &str = "teal_users";

pub async fn create_user(data: &User) -> anyhow::Result<bool> {
    let db = dynamo::get_global_db();
    let user_json = serde_json::to_value(data)?;
    db.put(TABLE_NAME, user_json).await
}

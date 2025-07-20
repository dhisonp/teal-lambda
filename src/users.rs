use crate::{
    dynamo::{self, USERS_TABLE_NAME},
    schema::User,
};

pub async fn create_user(data: &User) -> anyhow::Result<bool> {
    let db = dynamo::use_db();
    let user_json = serde_json::to_value(data)?;
    db.put(USERS_TABLE_NAME, user_json).await
}

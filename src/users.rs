use serde_json::to_value;

use crate::{
    dynamo::{use_db, USERS_TABLE_NAME},
    schema::User,
};

pub async fn create_user(data: &User) -> anyhow::Result<bool> {
    let db = use_db();
    db.put(USERS_TABLE_NAME, to_value(data)?).await
}

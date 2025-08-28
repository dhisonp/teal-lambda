use crate::dynamo::{use_db, TELLS_TABLE_NAME};
use crate::gemini::get_tell_response;
use crate::prompts;
use crate::schema::Context;
use chrono::Utc;
use serde::Serialize;
use serde_json::to_value;
use uuid::Uuid;

// TODO: Check if this is better to be defined in another module.
#[derive(Serialize)]
pub struct TellItem {
    pub tid: String,
    pub username: String, // Current user identifier. Should we replace with something else?
    pub tell: String,
    pub answer: String,
    pub user_state: String,
    pub mood: String,
    pub created_at: chrono::DateTime<Utc>,
    pub summary: Option<String>,
}

/// Tells Teal what the user is feeling, and Teal will return with a very benevolent response–
/// like the color teal! Optionally takes `context` for now, but this shouldn't be needed in most
/// cases.
pub async fn tell(username: &str, tell: &str, context: Option<Context>) -> anyhow::Result<String> {
    let context = context.unwrap_or_else(|| get_context()).to_string();
    let prompt_data = prompts::PromptData::Tell(prompts::TellReplacements {
        username,
        context: &context,
        tell,
    });
    let prompt = prompts::get_templated_prompt(prompts::PromptName::Tell, prompt_data)?;

    let response = get_tell_response(&prompt).await?;

    let data = TellItem {
        tid: Uuid::new_v4().to_string(),
        username: username.to_string(),
        tell: tell.to_string(),
        answer: response.answer.clone(),
        user_state: response.user_state.clone(),
        mood: response.mood.clone(),
        created_at: chrono::Utc::now(),
        summary: Some(response.summary.clone()),
    };

    let db = use_db();
    db.put(TELLS_TABLE_NAME, to_value(data)?).await?;

    Ok(response.answer)
}

/// Generate a Context object to be passed into tell() from the database.
// TODO: Adjust to new tell structure and optimize storage.
fn get_context() -> Context {
    Context {
        mood: "satisfied".to_string(),
        summary:
            "User shares job search frustrations but has new potential opportunity through family."
                .to_string(),
        summary_history: vec![
            "Hopeful, determined, but anxious about not messing up the opportunity.".to_string(),
            "User was feeling overwhelmed about work-life balance".to_string(),
            "User expressed excitement about a new project but worried about time management"
                .to_string(),
            "User felt confident after completing a challenging task".to_string(),
        ],
        tell_history: vec![
            "Another day of no job. But my uncle just sent me a text that his company may be hiring new engineers, and it may be a senior role. This time, I have to be strong. There is no way I can fumble this up.".to_string(),
            "I think while growth come with doubt, I'm feeling happy and there will be some potential interviews I'll be going this week.".to_string(),
            "You've successfully handled similar challenges before. A job will come to you if you truly believe in your own work.".to_string(),
            "It's getting tough. I'm confident and I know I can deliver, but why am I not getting jobs? It's becoming tough, to be fair.".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_context() {
        let context = get_context();

        assert_eq!(context.mood, "satisfied");
        assert!(context.summary.contains("job search frustrations"));
        assert_eq!(context.summary_history.len(), 4);
        assert_eq!(context.tell_history.len(), 4);

        assert!(context.summary_history[0].contains("Hopeful, determined"));
        assert!(context.tell_history[0].contains("Another day of no job"));
    }
}

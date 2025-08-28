use crate::dynamo::{use_db, TELLS_TABLE_NAME};
use crate::gemini::{ask_gemini, GeminiTellResponse};
use crate::prompts;
use chrono::Utc;
use serde::Serialize;
use serde_json::to_value;
use std::fmt;
use uuid::Uuid;

pub struct Context {
    pub mood: String,
    pub summary: String,
    pub summary_history: Vec<String>,
    pub tell_history: Vec<String>,
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "My current mood: {}. My current situation: {}. My past situations: {}. My past tells to you: {}.",
        self.mood,
        self.summary,
        self.summary_history.join(", "),
        self.tell_history.join(", "),
        )
    }
}

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
pub async fn tell(
    username: &str,
    user_message: &str,
    context: Option<Context>,
) -> anyhow::Result<String> {
    let context_string = context.unwrap_or_else(get_context).to_string();
    let prompt_data = prompts::PromptData::Tell(prompts::TellReplacements {
        username,
        context: &context_string,
        tell: user_message,
    });

    let prompt = prompts::create_prompt(prompts::PromptName::Tell, prompt_data)?;
    let response = ask_gemini(&prompt).await?;

    let tell_record = build_tell_record(username, user_message, &response);
    let db = use_db();
    db.put(TELLS_TABLE_NAME, to_value(tell_record)?).await?;

    Ok(response.answer)
}

/// Creates a TellItem from user input and AI response data. This is a pure function that can be
/// easily unit tested.
pub fn build_tell_record(
    username: &str,
    user_message: &str,
    ai_response: &GeminiTellResponse,
) -> TellItem {
    TellItem {
        tid: Uuid::new_v4().to_string(),
        username: username.to_string(),
        tell: user_message.to_string(),
        answer: ai_response.answer.clone(),
        user_state: ai_response.user_state.clone(),
        mood: ai_response.mood.clone(),
        created_at: Utc::now(),
        summary: Some(ai_response.summary.clone()),
    }
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

    #[test]
    fn test_context_display() {
        let context = Context {
            mood: "excited".to_string(),
            summary: "User got a new job".to_string(),
            summary_history: vec![
                "Was looking for work".to_string(),
                "Had interviews".to_string(),
            ],
            tell_history: vec![
                "I'm job hunting".to_string(),
                "Interview went well".to_string(),
            ],
        };

        let display = format!("{}", context);
        assert!(display.contains("My current mood: excited"));
        assert!(display.contains("My current situation: User got a new job"));
        assert!(display.contains("Was looking for work, Had interviews"));
        assert!(display.contains("I'm job hunting, Interview went well"));
    }

    #[test]
    fn test_context_display_empty_histories() {
        let context = Context {
            mood: "calm".to_string(),
            summary: "First conversation".to_string(),
            summary_history: vec![],
            tell_history: vec![],
        };

        let display = format!("{}", context);
        assert!(display.contains("My current mood: calm"));
        assert!(display.contains("My current situation: First conversation"));
        assert!(display.contains("My past situations: "));
        assert!(display.contains("My past tells to you: "));
    }

    #[test]
    fn test_context_display_single_items() {
        let context = Context {
            mood: "hopeful".to_string(),
            summary: "User shared good news".to_string(),
            summary_history: vec!["Previous summary".to_string()],
            tell_history: vec!["My single tell".to_string()],
        };

        let display = format!("{}", context);
        assert!(display.contains("Previous summary"));
        assert!(display.contains("My single tell"));
        assert!(!display.contains(", "));
    }

    #[test]
    fn test_build_tell_record() {
        use crate::gemini::GeminiTellResponse;

        let ai_response = GeminiTellResponse {
            answer: "That sounds like an exciting opportunity!".to_string(),
            summary: "User got job interview".to_string(),
            user_state: "hopeful and nervous".to_string(),
            mood: "excited".to_string(),
        };

        let tell_item =
            build_tell_record("testuser", "I have an interview tomorrow!", &ai_response);

        assert_eq!(tell_item.username, "testuser");
        assert_eq!(tell_item.tell, "I have an interview tomorrow!");
        assert_eq!(
            tell_item.answer,
            "That sounds like an exciting opportunity!"
        );
        assert_eq!(tell_item.user_state, "hopeful and nervous");
        assert_eq!(tell_item.mood, "excited");
        assert_eq!(
            tell_item.summary,
            Some("User got job interview".to_string())
        );

        // Verify UUID format (should be valid UUID string)
        assert!(!tell_item.tid.is_empty());
        assert!(tell_item.tid.contains('-'));

        // Verify timestamp is recent (within last minute)
        let now = chrono::Utc::now();
        let diff = now.signed_duration_since(tell_item.created_at);
        assert!(diff.num_seconds() < 60);
    }

    #[test]
    fn test_build_tell_record_with_empty_values() {
        use crate::gemini::GeminiTellResponse;

        let ai_response = GeminiTellResponse {
            answer: "".to_string(),
            summary: "".to_string(),
            user_state: "".to_string(),
            mood: "".to_string(),
        };

        let tell_item = build_tell_record("", "", &ai_response);

        assert_eq!(tell_item.username, "");
        assert_eq!(tell_item.tell, "");
        assert_eq!(tell_item.answer, "");
        assert_eq!(tell_item.user_state, "");
        assert_eq!(tell_item.mood, "");
        assert_eq!(tell_item.summary, Some("".to_string()));
    }
}

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
    pub mood: String,                 // TODO: Define set of moods
    pub summary: String,              // A summary of the user's current state of mind
    pub summary_history: Vec<String>, // History of past summaries
    pub tell_history: Vec<String>,    // History of past Tells
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
}

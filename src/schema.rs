use serde::Serialize;
use std::fmt;

// TODO: Storing OAuth2.0 credentials
#[derive(Serialize)]
pub struct User {
    pub tealant_id: String,
    pub name: String,
    pub email: String,
    pub current_mood: Option<Mood>,
    pub created_at: String, // TODO: Use chrono::DateTime<Utc>
}

#[derive(Debug, Serialize)]
pub enum Mood {
    Contemplative,
    // TODO: Add more along the way
}

impl fmt::Display for Mood {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Context {
    pub mood: Mood,                   // TODO: Define set of moods
    pub summary: String,              // A summary of the user's current state of mind
    pub summary_history: Vec<String>, // History of past summaries
    pub tell_history: Vec<String>,    // History of past Tells
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "My current mood: {}. My current situation: {}. My past situations: {}. My past tells to you: {}.",
        self.mood.to_string(),
        self.summary,
        self.summary_history.join(", "),
        self.tell_history.join(", "),
        )
    }
}

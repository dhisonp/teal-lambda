use crate::dynamo::{use_db, TELLS_TABLE_NAME};
use crate::schema::{self, Context};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::to_value;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiResponse {
    pub candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Candidate {
    pub content: Content,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Content {
    pub parts: Vec<Part>,
}

#[derive(Deserialize)]
struct Part {
    pub text: String,
}

#[derive(Deserialize)]
struct CombinedGeminiResponse {
    pub answer: String,
    pub summary: String,
    pub user_state: String,
}

// TODO: Check if this is better to be defined in schema.rs
#[derive(Serialize)]
struct TellItem {
    pub tid: String,
    pub username: String, // Should we use UIID instead of email/username here?
    pub answer: String,
    pub user_state: String,
    pub created_at: chrono::DateTime<Utc>,
    pub summary: Option<String>,
}

/// Receives a prompt argument and returns a direct reply from Gemini.
// NOTE: Recently modified to return a CombinedGeminiResponse instance. Rename as the purpose
// of the function has changed.
async fn ask_gemini(prompt: &str) -> anyhow::Result<CombinedGeminiResponse> {
    let url = format!(
        "{}?key={}",
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent",
        std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set")
    );

    let data = serde_json::json!({
        "contents": [
            {
                "parts": [
                    {
                        "text": &prompt,
                    }
                ]
            }
        ],
        "system_instruction": {
            "parts": [
                {
                    "text": "Speak an assertive, yet encouraging and soft-spoken, as if you're a therapist talking to a perfectly sane and healthy adult. Do not ask questions, and be concise and decisive with your answers."
                }
            ]
        },
        "generationConfig": {
            "temperature": 0.5,
            "maxOutputTokens": 500,
        }
    });

    let client = reqwest::Client::new();
    let res = client.post(&url).json(&data).send().await?;
    let body: GeminiResponse = res.json().await?;

    let mut text = body
        .candidates
        .as_ref()
        .and_then(|c| c.first())
        .and_then(|c| c.content.parts.first())
        .map(|p| p.text.as_str())
        .unwrap_or(
            r#"{
            "answer": "Gemini is not in a mood today",
            "summary": "No summary",
            "user_state": "No state"
        }"#,
        )
        .to_string();

    // Strip Markdown code block delimiters to ensure successful JSON parsing
    if text.starts_with("```json\n") && text.ends_with("\n```") {
        text = text.strip_prefix("```json\n").unwrap_or(&text).to_string();
        text = text.strip_suffix("\n```").unwrap_or(&text).to_string();
    }

    Ok(serde_json::from_str(&text)?)
}

/// Tells Teal what the user is feeling, and Teal will return with a very benevolent response– like the color teal!
/// Optionally takes `context` for now, but this shouldn't be needed in most cases.
pub(crate) async fn tell(
    username: &str,
    tell: &str,
    context: Option<Context>,
) -> anyhow::Result<String> {
    let context = context.unwrap_or_else(|| get_context());
    let prompt = format!(
        "My name is {}. Here is a context of my past conversations with you: {} (if I sent you no context, then this is our first conversation!). However, I have something to tell you about. {}.\n\nPlease provide your benevolent response to my tell, a concise third-person summary of my tell (max 12 words), and a concise summary of my current state of mind based on our conversation history and my latest tell (max 12 words).\n\nFormat your response as a JSON object with the following keys:\n- `answer`: Your benevolent response.\n- `summary`: A concise third-person summary of my tell, limited to 12 words.\n- `user_state`: A concise summary of my current state of mind, limited to 12 words.\n\nExample JSON format:\n```json\n{{\n  \"answer\": \"Your benevolent response here.\",\n  \"summary\": \"User expressed feelings about X.\",\n  \"user_state\": \"User is feeling Y.\"\n}}
```\nRemember to speak assertively, yet encouragingly and soft-spoken, like a therapist. Do not ask questions, and be concise and decisive with your answers.",
        username,
        context.to_string(),
        tell,
    );

    let result = ask_gemini(&prompt).await?;
    let answer = result.answer;
    let user_state = result.user_state;
    let summary = result.summary; // Do we need this? Review as we collect data.

    let data = TellItem {
        tid: Uuid::new_v4().to_string(),
        username: username.to_string(),
        answer: answer.clone(),
        user_state: user_state.clone(),
        created_at: chrono::Utc::now(),
        summary: Some(summary.clone()),
    };

    let db = use_db();
    db.put(TELLS_TABLE_NAME, to_value(data)?).await?;

    Ok(answer)
}

/// Generate a Context object to be passed into tell() from the database.
fn get_context() -> Context {
    Context {
            mood: schema::Mood::Contemplative,
            summary: "User is currently happy, albeit with some doubts on his career.".to_string(),
            summary_history: vec![
                "User was feeling overwhelmed about work-life balance".to_string(),
                "User expressed excitement about a new project but worried about time management"
                    .to_string(),
                "User felt confident after completing a challenging task".to_string(),
            ],
            tell_history: vec![
                "I think while growth come with doubt, I'm feeling happy and there will be some potential interviews I'll be going this week.".to_string(),
                "You've successfully handled similar challenges before. A job will come to you if you truly believe in your own work.".to_string(),
                "It's getting tough. I'm confident and I know I can deliver, but why am I not getting jobs? It's becoming tough, to be fair.".to_string(),
            ],
    }
}

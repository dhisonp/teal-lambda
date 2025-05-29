use dotenvy::Error;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiResponse {
    pub candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub content: Content,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub parts: Vec<Part>,
}

#[derive(Deserialize)]
pub struct Part {
    pub text: String,
}

struct Context {
    pub mood: String,                 // TODO: Define set of moods
    pub summary: String,              // A summary of the user's current state of mind
    pub summary_history: Vec<String>, // History of past summaries
    pub tell_history: Vec<String>,    // History of past Tells
}

/// Receives a prompt argument and returns a direct reply from Gemini.
pub(crate) async fn ask_gemini(prompt: &str) -> Result<String, reqwest::Error> {
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
    let res = client
        .post(&url)
        .json(&data)
        .send()
        .await?
        .error_for_status()?;
    let body: GeminiResponse = res.json().await?;

    let text = body
        .candidates
        .as_ref()
        .and_then(|c| c.first())
        .and_then(|c| c.content.parts.first())
        .map(|p| p.text.as_str())
        .unwrap_or("Gemini is not in a mood today")
        .to_string();

    Ok(text)
}

pub(crate) fn tell(username: &str, tell: &str, context: Option<Content>) -> Result<String, Error> {
    Ok("".to_string())
}

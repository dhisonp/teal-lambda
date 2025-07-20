use crate::schema::{self, Context};
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

/// Receives a prompt argument and returns a direct reply from Gemini.
async fn ask_gemini(prompt: &str) -> anyhow::Result<String> {
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

/// Tells Teal what the user is feeling, and Teal will return with a very benevolent response– like the color teal!
/// Optionally takes `context` for now, but this shouldn't be needed in most cases.
pub(crate) async fn tell(
    username: &str,
    tell: &str,
    context: Option<Context>,
) -> anyhow::Result<String> {
    let context = context.unwrap_or_else(|| get_context());
    let prompt = format!(
        "My name is {}. Here is a context of my past conversations with you: {} (if I sent you no context, then this is our first conversation!). However, I have something to tell you about. {}. What do you think?",
        username,
        context.to_string(),
        tell,
    );

    let res = ask_gemini(&prompt).await?;
    let answer = res.to_string();

    // TODO: Save these two into database
    // TODO: Combine into one prompt
    // let summary = summarize_tell(&answer).await?;
    // let state = generate_state(None, Some(&summary)).await?;

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

// /// Generate a summarized sentence of a generated Tell.
// pub(crate) async fn summarize_tell(tell: &str) -> anyhow::Result<String> {
//     let prompt = format!(
//         "Please summarize this to a single, concise single sentence: {}. This is for record keeping purposes, so write in third-person. Limit to concisely 12 words.",
//         &tell
//     );
//     let res = ask_gemini(&prompt).await?;
//     Ok(res)
// }

// /// Evaluate the current state of the user based on existing or given context, optionally with the latest summary.
// pub(crate) async fn generate_state(
//     context: Option<Context>,
//     summary: Option<&str>,
// ) -> anyhow::Result<String> {
//     let prompt = format!(
//         "Please summarize the user's current state of mind, based on the history: {}. Also account their latest conversation summary to you, if any: {}. This is for record keeping, so do not reference the user. Limit concisely to 12 words.",
//         context.unwrap_or(get_context()).to_string(),
//         summary.unwrap_or("None for now!"),
//     );

//     let res = ask_gemini(&prompt).await?;
//     Ok(res)
// }

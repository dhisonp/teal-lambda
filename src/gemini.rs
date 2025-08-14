use crate::dynamo::{use_db, TELLS_TABLE_NAME};
use crate::prompts;
use crate::schema::Context;
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
struct GeminiTellResponse {
    pub answer: String,
    pub summary: String,
    pub user_state: String,
    pub mood: String,
}

// TODO: Check if this is better to be defined in another module.
#[derive(Serialize, Deserialize)]
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

/// Receives a prompt argument and returns a direct reply from Gemini.
// NOTE: Recently modified to return a CombinedGeminiResponse instance. Rename as the purpose
// of the function has changed.
async fn get_tell_response(prompt: &str) -> anyhow::Result<GeminiTellResponse> {
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
        .unwrap_or("Gemini is not in a mood today!")
        .to_string();

    // Strip Markdown code block delimiters to ensure successful JSON parsing
    if text.starts_with("```json\n") && text.ends_with("\n```") {
        text = text.strip_prefix("```json\n").unwrap_or(&text).to_string();
        text = text.strip_suffix("\n```").unwrap_or(&text).to_string();
    }

    Ok(serde_json::from_str(&text)?)
}

/// Tells Teal what the user is feeling, and Teal will return with a very benevolent response–
/// like the color teal! Optionally takes `context` for now, but this shouldn't be needed in most
/// cases.
pub(crate) async fn tell(
    username: &str,
    tell: &str,
    context: Option<Context>,
) -> anyhow::Result<String> {
    let context = if let Some(ctx) = context {
        ctx
    } else {
        get_context(username).await
    };
    let context_string = context.to_string();
    let prompt_data = prompts::PromptData::Tell(prompts::TellReplacements {
        username,
        context: &context_string,
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
// NOTE: Should we return a Result?
async fn get_context(username: &str) -> Context {
    let tells = use_db()
        .get_tells_by_username(username)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Error fetching tells for context: {}", e);
            Vec::new()
        });

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

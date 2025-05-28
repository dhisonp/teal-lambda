use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiResponse {
    pub candidates: Vec<Candidate>,
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
pub(crate) async fn ask_gemini(prompt: &str) -> Result<String, reqwest::Error> {
    let url = format!(
        "{}?key={}",
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent",
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
        ]
    });

    let client = reqwest::Client::new();
    let res = client.post(&url).json(&data).send().await?;
    let body: GeminiResponse = res.json().await?;
    let text = body
        .candidates
        .first()
        .and_then(|c| c.content.parts.first())
        .map(|p| p.text.as_str())
        .unwrap_or("Gemini is not in the mood today.");

    Ok(text.to_string())
}

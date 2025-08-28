use serde::Deserialize;

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
pub struct GeminiTellResponse {
    pub answer: String,
    pub summary: String,
    pub user_state: String,
    pub mood: String,
}

/// Receives a prompt argument and returns a direct reply from Gemini.
pub async fn ask_gemini(prompt: &str) -> anyhow::Result<GeminiTellResponse> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_gemini_tell_response_deserialization() {
        let json_data = json!({
            "answer": "That sounds challenging, but you're handling it well.",
            "summary": "User facing work challenges",
            "user_state": "determined but stressed",
            "mood": "anxious"
        });

        let response: GeminiTellResponse = serde_json::from_value(json_data).unwrap();
        assert_eq!(
            response.answer,
            "That sounds challenging, but you're handling it well."
        );
        assert_eq!(response.summary, "User facing work challenges");
        assert_eq!(response.user_state, "determined but stressed");
        assert_eq!(response.mood, "anxious");
    }

    #[test]
    fn test_markdown_code_block_stripping() {
        let mut text = "```json\n{\"answer\": \"test\"}\n```".to_string();

        if text.starts_with("```json\n") && text.ends_with("\n```") {
            text = text.strip_prefix("```json\n").unwrap_or(&text).to_string();
            text = text.strip_suffix("\n```").unwrap_or(&text).to_string();
        }

        assert_eq!(text, "{\"answer\": \"test\"}");
    }

    #[test]
    fn test_gemini_response_extraction() {
        let json_data = json!({
            "candidates": [
                {
                    "content": {
                        "parts": [
                            {
                                "text": "```json\n{\"answer\":\"Hello\",\"summary\":\"Test\",\"user_state\":\"good\",\"mood\":\"happy\"}\n```"
                            }
                        ]
                    }
                }
            ]
        });

        let response: GeminiResponse = serde_json::from_value(json_data).unwrap();
        let mut extracted_text = response
            .candidates
            .as_ref()
            .and_then(|c| c.first())
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.as_str())
            .unwrap_or("Gemini is not in a mood today!")
            .to_string();

        if extracted_text.starts_with("```json\n") && extracted_text.ends_with("\n```") {
            extracted_text = extracted_text
                .strip_prefix("```json\n")
                .unwrap_or(&extracted_text)
                .to_string();
            extracted_text = extracted_text
                .strip_suffix("\n```")
                .unwrap_or(&extracted_text)
                .to_string();
        }

        let tell_response: Result<GeminiTellResponse, _> = serde_json::from_str(&extracted_text);
        assert!(tell_response.is_ok());

        let tell_response = tell_response.unwrap();
        assert_eq!(tell_response.answer, "Hello");
        assert_eq!(tell_response.summary, "Test");
        assert_eq!(tell_response.user_state, "good");
        assert_eq!(tell_response.mood, "happy");
    }
}

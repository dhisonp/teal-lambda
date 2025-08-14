use include_dir::{include_dir, Dir};
use std::collections::HashMap;

static PROMPTS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/prompts");

pub enum PromptName {
    Tell,
}

impl PromptName {
    pub fn as_str(&self) -> &'static str {
        match self {
            PromptName::Tell => "tell.md",
        }
    }
}

pub fn get_templated_prompt(
    prompt_name: PromptName,
    replacements: &HashMap<&str, &str>,
) -> anyhow::Result<String> {
    let template = PROMPTS_DIR
        .get_file(prompt_name.as_str())
        .ok_or_else(|| anyhow::anyhow!("Prompt template '{}' not found", prompt_name.as_str()))?
        .contents_utf8()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid UTF-8 in prompt template '{}'",
                prompt_name.as_str()
            )
        })?;

    let mut prompt = template.to_string();
    for (key, value) in replacements {
        prompt = prompt.replace(&format!("{{{}}}", key), value);
    }
    Ok(prompt)
}

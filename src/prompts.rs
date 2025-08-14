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

pub struct TellReplacements<'a> {
    pub username: &'a str,
    pub context: &'a str,
    pub tell: &'a str,
}

pub enum PromptData<'a> {
    Tell(TellReplacements<'a>),
}

pub fn get_templated_prompt(prompt_name: PromptName, data: PromptData) -> anyhow::Result<String> {
    let filename = prompt_name.as_str();
    let template = PROMPTS_DIR
        .get_file(filename)
        .ok_or_else(|| anyhow::anyhow!("Prompt template '{}' not found", filename))?
        .contents_utf8()
        .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in prompt template '{}'", filename))?;

    let replacements_map = match data {
        PromptData::Tell(tell_data) => {
            let mut map = HashMap::new();
            map.insert("username", tell_data.username);
            map.insert("context", tell_data.context);
            map.insert("tell", tell_data.tell);
            map
        }
    };

    let mut prompt = template.to_string();
    for (key, value) in &replacements_map {
        prompt = prompt.replace(&format!("{{{}}}", key), value);
    }
    Ok(prompt)
}

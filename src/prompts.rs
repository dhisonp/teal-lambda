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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_name_as_str() {
        assert_eq!(PromptName::Tell.as_str(), "tell.md");
    }

    #[test]
    fn test_get_templated_prompt_tell() {
        let tell_data = TellReplacements {
            username: "testuser",
            context: "User was feeling happy yesterday",
            tell: "I had a great day today!",
        };
        let data = PromptData::Tell(tell_data);
        
        let result = get_templated_prompt(PromptName::Tell, data);
        assert!(result.is_ok());
        
        let prompt = result.unwrap();
        assert!(prompt.contains("testuser"));
        assert!(prompt.contains("User was feeling happy yesterday"));
        assert!(prompt.contains("I had a great day today!"));
        assert!(!prompt.contains("{username}"));
        assert!(!prompt.contains("{context}"));
        assert!(!prompt.contains("{tell}"));
    }

    #[test]
    fn test_get_templated_prompt_empty_values() {
        let tell_data = TellReplacements {
            username: "",
            context: "",
            tell: "",
        };
        let data = PromptData::Tell(tell_data);
        
        let result = get_templated_prompt(PromptName::Tell, data);
        assert!(result.is_ok());
        
        let prompt = result.unwrap();
        assert!(!prompt.contains("{username}"));
        assert!(!prompt.contains("{context}"));
        assert!(!prompt.contains("{tell}"));
    }

    #[test]
    fn test_get_templated_prompt_with_special_characters() {
        let tell_data = TellReplacements {
            username: "user@test.com",
            context: "User said: \"I'm feeling great!\"",
            tell: "Today I achieved 100% on my test & I'm happy!",
        };
        let data = PromptData::Tell(tell_data);
        
        let result = get_templated_prompt(PromptName::Tell, data);
        assert!(result.is_ok());
        
        let prompt = result.unwrap();
        assert!(prompt.contains("user@test.com"));
        assert!(prompt.contains("User said: \"I'm feeling great!\""));
        assert!(prompt.contains("Today I achieved 100% on my test & I'm happy!"));
    }
}

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde_json::json;
use std::env;

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_MODEL: &str = "gpt-5.4-mini";
const DEFAULT_GOOGLE_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/openai";
const DEFAULT_GOOGLE_MODEL: &str = "gemini-3.5-flash";
const SYSTEM_PROMPT: &str = "Generate one concise Conventional Commit message. Use the format type(scope): description. Keep it under about 72 characters. Return only the commit message, with no explanation and no markdown.";

#[derive(Debug, Clone)]
pub struct AiConfig {
    pub provider: Provider,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    OpenAi,
    Google,
}

impl Provider {
    fn as_str(self) -> &'static str {
        match self {
            Provider::OpenAi => "openai",
            Provider::Google => "google",
        }
    }
}

impl AiConfig {
    pub fn from_env() -> Result<Self> {
        Self::from_env_with(|key| env::var(key).ok())
    }

    pub(crate) fn from_env_with<F>(mut get_env: F) -> Result<Self>
    where
        F: FnMut(&str) -> Option<String>,
    {
        let provider = match get_env("AI_PROVIDER")
            .as_deref()
            .map(|value| value.to_ascii_lowercase())
        {
            Some(value) if value == "openai" => Provider::OpenAi,
            Some(value) if value == "google" => Provider::Google,
            Some(value) => bail!("unsupported AI_PROVIDER value: {value}"),
            None => detect_provider(&mut get_env)?,
        };

        let (api_key, base_url, model) = match provider {
            Provider::OpenAi => {
                let api_key = first_env(&mut get_env, &["OPENAI_API_KEY"])
                    .context("OPENAI_API_KEY is required")?;
                let base_url = first_env(&mut get_env, &["OPENAI_BASE_URL"])
                    .unwrap_or_else(|| DEFAULT_BASE_URL.to_owned());
                let model = first_env(&mut get_env, &["OPENAI_MODEL"])
                    .unwrap_or_else(|| DEFAULT_MODEL.to_owned());
                (api_key, base_url, model)
            }
            Provider::Google => {
                let api_key = first_env(&mut get_env, &["GOOGLE_API_KEY", "GEMINI_API_KEY"])
                    .context("GOOGLE_API_KEY is required")?;
                let base_url = first_env(&mut get_env, &["GOOGLE_BASE_URL", "GEMINI_BASE_URL"])
                    .unwrap_or_else(|| DEFAULT_GOOGLE_BASE_URL.to_owned());
                let model = first_env(&mut get_env, &["GOOGLE_MODEL", "GEMINI_MODEL"])
                    .unwrap_or_else(|| DEFAULT_GOOGLE_MODEL.to_owned());
                (api_key, base_url, model)
            }
        };

        Ok(Self {
            provider,
            api_key,
            base_url,
            model,
        })
    }
}

pub async fn suggest_commit_message(
    config: &AiConfig,
    diff: &str,
    verbose: bool,
) -> Result<String> {
    let suggestions = suggest_commit_messages(config, diff, 1, verbose).await?;
    suggestions
        .into_iter()
        .next()
        .context("AI API response did not include a commit message")
}

pub async fn suggest_commit_messages(
    config: &AiConfig,
    diff: &str,
    count: usize,
    verbose: bool,
) -> Result<Vec<String>> {
    if count == 0 {
        bail!("suggestion count must be greater than zero");
    }

    let endpoint = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();

    if verbose {
        eprintln!("ai provider: {}", config.provider.as_str());
        eprintln!("ai endpoint: {endpoint}");
        eprintln!("ai model: {}", config.model);
    }

    let payload = json!({
        "model": config.model,
        "messages": [
            {
                "role": "system",
                "content": SYSTEM_PROMPT
            },
            {
                "role": "user",
                "content": format!(
                    "Staged git diff:\n\n{diff}\n\nGenerate exactly {count} distinct Conventional Commit suggestions. Return one suggestion per line with no numbering, no markdown, and no explanation."
                )
            }
        ],
        "temperature": 0.2
    });

    let response = client
        .post(&endpoint)
        .bearer_auth(&config.api_key)
        .json(&payload)
        .send()
        .await
        .context("failed to contact the AI API")?;

    if verbose {
        eprintln!("ai status: {}", response.status());
    }

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        bail!("AI API request failed ({status}): {body}");
    }

    let body: ChatCompletionResponse = response
        .json()
        .await
        .context("failed to parse AI API response")?;

    let message = body
        .choices
        .first()
        .and_then(|choice| choice.message.as_ref())
        .and_then(|message| message.content.as_deref())
        .context("AI API response did not include a commit message")?;

    let suggestions = message
        .lines()
        .map(clean_ai_line)
        .filter(|line| !line.is_empty())
        .map(|line| {
            line.trim_matches(|ch| ch == '"' || ch == '\'')
                .trim()
                .to_owned()
        })
        .filter(|line| !line.is_empty())
        .take(count)
        .collect::<Vec<_>>();

    if suggestions.is_empty() {
        bail!("the AI API returned no usable commit message suggestions");
    }

    if suggestions.len() < count {
        bail!(
            "the AI API returned only {} suggestion(s); expected {}",
            suggestions.len(),
            count
        );
    }

    Ok(suggestions)
}

fn clean_ai_line(line: &str) -> String {
    let trimmed = line.trim();
    if let Some(stripped) = trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))
    {
        return stripped.to_owned();
    }

    let mut marker_end = 0;
    for (index, character) in trimmed.char_indices() {
        if character.is_ascii_digit() {
            marker_end = index + character.len_utf8();
            continue;
        }

        if matches!(character, '.' | ')') {
            let rest = trimmed[index + character.len_utf8()..].trim_start();
            if !rest.is_empty() {
                return rest.to_owned();
            }
        }

        break;
    }

    trimmed[marker_end..].trim_start().to_owned()
}

fn detect_provider<F>(get_env: &mut F) -> Result<Provider>
where
    F: FnMut(&str) -> Option<String>,
{
    if first_env(get_env, &["OPENAI_API_KEY"]).is_some() {
        return Ok(Provider::OpenAi);
    }

    if first_env(get_env, &["GOOGLE_API_KEY", "GEMINI_API_KEY"]).is_some() {
        return Ok(Provider::Google);
    }

    bail!("set OPENAI_API_KEY or GOOGLE_API_KEY and optionally AI_PROVIDER to choose a provider")
}

fn first_env<F>(get_env: &mut F, names: &[&str]) -> Option<String>
where
    F: FnMut(&str) -> Option<String>,
{
    names.iter().find_map(|name| get_env(name))
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Option<ChatMessage>,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{AiConfig, Provider};
    use std::collections::HashMap;

    fn config_from(map: HashMap<&str, &str>) -> AiConfig {
        AiConfig::from_env_with(|key| map.get(key).map(|value| value.to_string())).unwrap()
    }

    #[test]
    fn prefers_openai_by_default_when_openai_key_exists() {
        let config = config_from(HashMap::from([("OPENAI_API_KEY", "openai-key")]));
        assert_eq!(config.provider, Provider::OpenAi);
        assert_eq!(config.base_url, "https://api.openai.com/v1");
        assert_eq!(config.model, "gpt-5.4-mini");
    }

    #[test]
    fn configures_google_when_requested() {
        let config = config_from(HashMap::from([
            ("AI_PROVIDER", "google"),
            ("GOOGLE_API_KEY", "google-key"),
        ]));

        assert_eq!(config.provider, Provider::Google);
        assert_eq!(
            config.base_url,
            "https://generativelanguage.googleapis.com/v1beta/openai"
        );
        assert_eq!(config.model, "gemini-3.5-flash");
    }

    #[test]
    fn uses_google_aliases() {
        let config = config_from(HashMap::from([
            ("GOOGLE_API_KEY", "google-key"),
            ("GEMINI_MODEL", "gemini-2.5-flash"),
        ]));

        assert_eq!(config.provider, Provider::Google);
        assert_eq!(config.model, "gemini-2.5-flash");
    }

    #[test]
    fn rejects_unknown_provider() {
        let error = AiConfig::from_env_with(|key| match key {
            "AI_PROVIDER" => Some("anthropic".to_string()),
            _ => None,
        })
        .unwrap_err();

        assert!(error.to_string().contains("unsupported AI_PROVIDER value"));
    }
}

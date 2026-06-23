use std::time::{Duration, Instant};

use crate::app_state::ProviderConfig;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct TranslationPayload {
    pub source: String,
    pub target: String,
    pub latency: u64,
    pub provider: String,
    pub source_lang: String,
    pub target_lang: String,
}

#[derive(Deserialize)]
struct MyMemoryResponse {
    #[serde(rename = "responseData")]
    response_data: MyMemoryResponseData,
    #[serde(rename = "responseStatus")]
    response_status: i32,
}

#[derive(Deserialize)]
struct MyMemoryResponseData {
    #[serde(rename = "translatedText")]
    translated_text: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    stream: bool,
    thinking: Option<ThinkingConfig>,
}

#[derive(Serialize)]
struct ThinkingConfig {
    #[serde(rename = "type")]
    kind: &'static str,
}

#[derive(Serialize)]
struct AnthropicMessagesRequest {
    model: String,
    system: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: &'static str,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicMessagesResponse {
    content: Vec<AnthropicContentBlock>,
}

#[derive(Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    kind: String,
    text: Option<String>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: &'static str,
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

#[derive(Deserialize)]
struct ChatCompletionChoice {
    message: ChatCompletionMessage,
}

#[derive(Deserialize)]
struct ChatCompletionMessage {
    content: String,
}

pub async fn translate_text_with_provider(
    source: String,
    source_lang: String,
    target_lang: String,
    provider_config: &ProviderConfig,
) -> anyhow::Result<TranslationPayload> {
    let started_at = Instant::now();
    let trimmed = source.trim().to_string();

    if trimmed.is_empty() {
        anyhow::bail!("empty source text");
    }

    let langpair = resolve_langpair(&trimmed, &source_lang, &target_lang);
    let (target, provider) = if provider_config.protocol == "mymemory" {
        (
            translate_with_mymemory(&trimmed, langpair).await?,
            format!("mymemory:{}", langpair),
        )
    } else {
        (
            translate_with_chat_provider(&trimmed, langpair, provider_config).await?,
            format!("{}:{}", provider_config.name, provider_config.model),
        )
    };

    Ok(TranslationPayload {
        source: trimmed,
        target,
        latency: started_at.elapsed().as_millis() as u64,
        provider,
        source_lang: langpair.split('|').next().unwrap_or("auto").to_string(),
        target_lang: langpair.split('|').nth(1).unwrap_or("zh-CN").to_string(),
    })
}

async fn translate_with_chat_provider(
    source: &str,
    langpair: &str,
    config: &ProviderConfig,
) -> anyhow::Result<String> {
    let api_key = config
        .api_key
        .as_ref()
        .filter(|key| !key.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("AI provider api key is not configured"))?;
    match config.protocol.as_str() {
        "anthropic" => translate_with_anthropic_provider(source, langpair, config, api_key).await,
        _ => translate_with_openai_provider(source, langpair, config, api_key).await,
    }
}

async fn translate_with_openai_provider(
    source: &str,
    langpair: &str,
    config: &ProviderConfig,
    api_key: &str,
) -> anyhow::Result<String> {
    let endpoint = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()?;
    let (source_lang, target_lang) = split_langpair(langpair);
    let request = ChatCompletionRequest {
        model: config.model.clone(),
        temperature: 0.2,
        stream: false,
        thinking: Some(ThinkingConfig { kind: "disabled" }),
        messages: vec![
            ChatMessage {
                role: "system",
                content: config.agent_prompt.clone(),
            },
            ChatMessage {
                role: "user",
                content: format!("Translate from {source_lang} to {target_lang}. Text:\n{source}"),
            },
        ],
    };

    let response = client
        .post(endpoint)
        .bearer_auth(api_key)
        .json(&request)
        .send()
        .await?
        .error_for_status()?
        .json::<ChatCompletionResponse>()
        .await?;

    let translated = response
        .choices
        .first()
        .map(|choice| choice.message.content.trim().to_string())
        .filter(|text| !text.is_empty())
        .ok_or_else(|| anyhow::anyhow!("AI provider returned empty text"))?;

    Ok(translated)
}

async fn translate_with_anthropic_provider(
    source: &str,
    langpair: &str,
    config: &ProviderConfig,
    api_key: &str,
) -> anyhow::Result<String> {
    let base_url = config.base_url.trim_end_matches('/');
    let endpoint = if base_url.ends_with("/v1") {
        format!("{base_url}/messages")
    } else {
        format!("{base_url}/v1/messages")
    };
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()?;
    let (source_lang, target_lang) = split_langpair(langpair);
    let request = AnthropicMessagesRequest {
        model: config.model.clone(),
        system: config.agent_prompt.clone(),
        max_tokens: 2048,
        temperature: 0.2,
        messages: vec![AnthropicMessage {
            role: "user",
            content: format!("Translate from {source_lang} to {target_lang}. Text:\n{source}"),
        }],
    };

    let response = client
        .post(endpoint)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&request)
        .send()
        .await?
        .error_for_status()?
        .json::<AnthropicMessagesResponse>()
        .await?;

    let translated = response
        .content
        .iter()
        .filter(|block| block.kind == "text")
        .filter_map(|block| block.text.as_deref())
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string();

    if translated.is_empty() {
        anyhow::bail!("AI provider returned empty text");
    }

    Ok(translated)
}

fn split_langpair(langpair: &str) -> (&str, &str) {
    let mut parts = langpair.split('|');
    (
        parts.next().unwrap_or("auto"),
        parts.next().unwrap_or("zh-CN"),
    )
}

async fn translate_with_mymemory(source: &str, langpair: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()?;

    let response = client
        .get("https://api.mymemory.translated.net/get")
        .query(&[("q", source), ("langpair", langpair), ("mt", "1")])
        .send()
        .await?
        .error_for_status()?
        .json::<MyMemoryResponse>()
        .await?;

    if response.response_status != 200 {
        anyhow::bail!(
            "translation provider returned status {}",
            response.response_status
        );
    }

    let translated = response.response_data.translated_text.trim().to_string();
    if translated.is_empty() {
        anyhow::bail!("translation provider returned empty text");
    }

    Ok(translated)
}

fn resolve_langpair(source: &str, source_lang: &str, target_lang: &str) -> &'static str {
    match (source_lang, target_lang) {
        ("en", "zh-CN") => "en|zh-CN",
        ("zh-CN", "en") => "zh-CN|en",
        ("ja", "zh-CN") => "ja|zh-CN",
        ("zh-CN", "ja") => "zh-CN|ja",
        ("ko", "zh-CN") => "ko|zh-CN",
        ("zh-CN", "ko") => "zh-CN|ko",
        ("auto", "en") => {
            if source.chars().any(is_cjk) {
                "zh-CN|en"
            } else {
                "en|zh-CN"
            }
        }
        ("auto", "zh-CN") => {
            if source.chars().any(is_cjk) {
                "zh-CN|en"
            } else {
                "en|zh-CN"
            }
        }
        _ => {
            if source.chars().any(is_cjk) {
                "zh-CN|en"
            } else {
                "en|zh-CN"
            }
        }
    }
}

fn is_cjk(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF
    )
}

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{env, error::Error, io};
use tokio_stream::StreamExt;

type AppError = Box<dyn Error + Send + Sync>;

const AI_CHAT_URL: &str = "https://ai.hackclub.com/proxy/v1/chat/completions";

#[derive(Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
}

#[derive(Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

pub async fn fetch_ai_response(
    client: &Client,
    model: String,
    prompt: String,
) -> Result<String, AppError> {
    let api_key = api_key()?;

    let request = ChatRequest {
        model: model,
        messages: vec![Message {
            role: String::from("user"),
            content: prompt,
        }],
        stream: false,
    };

    let response = client
        .post(AI_CHAT_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?
        .error_for_status()?
        .json::<ChatResponse>()
        .await?;

    Ok(response.choices[0].message.content.clone())
}

pub async fn fetch_ai_response_stream<F>(
    client: &Client,
    model: String,
    prompt: String,
    on_token: F,
) -> Result<(), AppError>
where
    F: Fn(String) + Send + Sync,
{
    let api_key = api_key()?;

    let request = ChatRequest {
        model,
        messages: vec![Message {
            role: String::from("user"),
            content: prompt,
        }],
        stream: true,
    };

    let mut stream = client
        .post(AI_CHAT_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?
        .error_for_status()?
        .bytes_stream();

    let mut buffer = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        buffer.extend_from_slice(&chunk);

        while let Some(newline_pos) = buffer.iter().position(|byte| *byte == b'\n') {
            let line_bytes: Vec<u8> = buffer.drain(..=newline_pos).collect();
            if let Ok(line) = std::str::from_utf8(&line_bytes) {
                if parse_stream_line(line, &on_token) {
                    return Ok(());
                }
            }
        }
    }

    if !buffer.is_empty() {
        if let Ok(line) = std::str::from_utf8(&buffer) {
            parse_stream_line(line, &on_token);
        }
    }

    Ok(())
}

fn api_key() -> Result<String, AppError> {
    env::var("HACKCLUB_API_KEY")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "HACKCLUB_API_KEY must be set").into())
}

fn parse_stream_line<F>(line: &str, on_token: &F) -> bool
where
    F: Fn(String) + Send + Sync,
{
    let line = line.trim();

    let Some(data) = line.strip_prefix("data: ") else {
        return false;
    };

    if data == "[DONE]" {
        return true;
    }

    if let Ok(stream_resp) = serde_json::from_str::<serde_json::Value>(data) {
        if let Some(content) = stream_resp["choices"][0]["delta"]["content"].as_str() {
            on_token(content.to_string());
        }
    }

    false
}

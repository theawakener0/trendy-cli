use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use tokio_stream::StreamExt;

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

pub async fn fetch_ai_response(client: &Client,model: String, prompt: String) -> Result<String, reqwest::Error> {
    let api_key = env::var("HACKCLUB_API_KEY").expect("HACKCLUB_API_KEY must be set");

    let request = ChatRequest {
        model: model,
        messages: vec![Message {
            role: String::from("user"),
            content: prompt,
        }],
        stream: false,
    };

    let response = client
        .post("https://ai.hackclub.com/proxy/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?
        .json::<ChatResponse>()
        .await?;

    Ok(response.choices[0].message.content.clone())
}

pub async fn fetch_ai_response_stream<F>(client: &Client, model: String, prompt: String, on_token: F) -> Result<(), reqwest::Error>
where
    F: Fn(String) + Send + Sync,
{
    let api_key = env::var("HACKCLUB_API_KEY").expect("HACKCLUB_API_KEY must be set");

    let request = ChatRequest {
        model,
        messages: vec![Message {
            role: String::from("user"),
            content: prompt,
        }],
        stream: true,
    };

    let mut stream = client
        .post("https://ai.hackclub.com/proxy/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?
        .bytes_stream();

    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk: bytes::Bytes = chunk?;
        if let Ok(text) = String::from_utf8(chunk.to_vec()) {
            buffer.push_str(&text);
            
            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer.drain(..=newline_pos).collect::<String>();
                let line = line.trim();
                
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" {
                        return Ok(());
                    }
                    if let Ok(stream_resp) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(content) = stream_resp["choices"][0]["delta"]["content"].as_str() {
                            on_token(content.to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

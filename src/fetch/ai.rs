use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
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
    let api_key = env::var("HACKCLUB_API_KEY");

    let request = ChatRequest {
        model: model,
        messages: vec![Message {
            role: String::from("user"),
            content: prompt,
        }],
    };

    let response = client
        .post("https://ai.hackclub.com/proxy/v1/chat/completions")
        .header("Authorization", format!("Bearer {:?}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?
        .json::<ChatResponse>()
        .await?;

    Ok(response.choices[0].message.content.clone())
}

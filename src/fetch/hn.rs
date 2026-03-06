use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HNStory {
    pub title: Option<String>,
    pub url: Option<String>,
    pub score: Option<u32>,
}

pub async fn fetch_top_ids_hn(client: &Client) -> Result<Vec<u64>, reqwest::Error> {
    client
        .get("https://hacker-news.firebaseio.com/v0/topstories.json")
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<u64>>()
        .await
}

pub async fn fetch_story_hn(client: &Client, id: u64) -> Result<HNStory, reqwest::Error> {
    let url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
    client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json::<HNStory>()
        .await
}

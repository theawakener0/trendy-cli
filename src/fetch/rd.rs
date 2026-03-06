use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RedditResponse {
    pub data: RedditData,
}

#[derive(Debug, Deserialize)]
pub struct RedditData {
    pub children: Vec<RedditPost>,
}

#[derive(Debug, Deserialize)]
pub struct RedditPost {
    pub data: PostData,
}

#[derive(Debug, Deserialize)]
pub struct PostData {
    pub title: Option<String>,
    pub score: u32,
    pub url: Option<String>,
}

pub async fn fetch_from_subreddit(
    client: &Client,
    subreddit: String,
    lim: usize,
) -> Result<RedditResponse, reqwest::Error> {
    let url = format!(
        "https://www.reddit.com/r/{}/top.json?limit={}",
        subreddit, lim
    );
    client
        .get(&url)
        .header("User-Agent", "trendy-cli")
        .send()
        .await?
        .error_for_status()?
        .json::<RedditResponse>()
        .await
}

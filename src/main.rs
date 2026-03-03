use reqwest::Client;
use serde::Deserialize;


#[derive(Debug, Deserialize)]
struct HNStory {
    title: Option<String>,
    url: Option<String>,
    score: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct RedditResponse {
    data: RedditData,
}

#[derive(Debug, Deserialize)]
struct RedditData {
    children: Vec<RedditPost>,
}

#[derive(Debug, Deserialize)]
struct RedditPost {
    data: PostData,
}

#[derive(Debug, Deserialize)]
struct PostData {
    title: String,
    score: u32,
    url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let client = Client::new();

    banner();

    let response_hn = fetch_top_ids_hn(&client).await.unwrap();

    for id in response_hn.iter().take(10) {
        match fetch_story_hn(&client, *id).await {
            Ok(story) => {
                println!("| Title: {:.<26} |", story.title.as_deref().unwrap_or("N/A"));
                println!("| URL:   {:.<26} |", story.url.as_deref().unwrap_or("N/A"));
                println!("| Score: {:.<26} |", story.score.map(|s| s.to_string()).unwrap_or_else(|| "N/A".to_string()));
                println!("| {:-<30} ", "");
            }
            Err(e) => {
                eprintln!("Failed to fetch story {}: {}", id, e);
            }
        }
    }

    let response_rd = fetch_from_subreddit(&client, "rust", 10).await.unwrap();

    for post in response_rd.data.children.iter() {
        println!("| Title: {:.<26} |", post.data.title);
        println!("| URL:   {:.<26} |", post.data.url);
        println!("| UpVotes: {:.<26} |", post.data.score);
        println!("| {:-<30} ", "");
    }


    
    Ok(())

}

fn banner() {

    const RESET: &str = "\x1b[0m";
    const CLEAR: &str = "\x1b[2J\x1b[H";
    const ORANGE: &str = "\x1b[38;2;255;165;0m";

    let banner = r#"
                                                                     
▄▄▄▄▄▄▄▄▄                   ▄▄        ▄▄▄▄▄▄▄ ▄▄▄      ▄▄▄▄▄ 
▀▀▀███▀▀▀                   ██       ███▀▀▀▀▀ ███       ███  
   ███ ████▄ ▄█▀█▄ ████▄ ▄████ ██ ██ ███      ███       ███  
   ███ ██ ▀▀ ██▄█▀ ██ ██ ██ ██ ██▄██ ███      ███       ███  
   ███ ██    ▀█▄▄▄ ██ ██ ▀████  ▀██▀ ▀███████ ████████ ▄███▄ 
                                 ██                          
                               ▀▀▀                           
    "#;

    println!("{}", CLEAR);
    println!("{} {} {}",ORANGE, banner, RESET);

}


async fn fetch_top_ids_hn(client: &Client) -> Result<Vec<u64>, reqwest::Error> {
        client
        .get("https://hacker-news.firebaseio.com/v0/topstories.json")
        .send()
        .await?
        .json::<Vec<u64>>()
        .await
}

async fn fetch_story_hn(client: &Client, id: u64) -> Result<HNStory, reqwest::Error> {
    let url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
    client
        .get(&url)
        .send()
        .await?
        .json::<HNStory>()
        .await
}

async fn fetch_from_subreddit(client: &Client, subreddit: &str, lim: u32) -> Result<RedditResponse, reqwest::Error> {
    let url = format!("https://www.reddit.com/r/{}/top.json?limit={}", subreddit, lim);
    client
        .get(&url)
        .header("User-Agent", "trendy-cli")
        .send()
        .await?
        .json::<RedditResponse>()
        .await
}





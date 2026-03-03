use reqwest::Client;
use serde::Deserialize;


#[derive(Debug, Deserialize)]
struct APIResponse {
    title: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let client = Client::new();

    banner();

    let response = fetch_top_ids_hn(&client).await.unwrap();

    println!("{:#?}", response);
    
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







use reqwest::Client;
use clap::{Parser};
use crate::fetch::hn::{fetch_top_ids_hn, fetch_story_hn};
use crate::fetch::rd::{fetch_from_subreddit};

pub mod fetch; 


const RESET: &str = "\x1b[0m";
const CLEAR: &str = "\x1b[2J\x1b[H";
const ORANGE: &str = "\x1b[38;2;255;165;0m";
const RED: &str = "\x1b[0;31m";


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 10)]
    limit: usize,
    
    #[arg(short, long, default_value = "rust")]
    subreddit: String,

    #[arg(short='h', long="hn")]
    hn_flag: bool,

    #[arg(short='r', long="rd")]
    rd_flag: bool,


}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let client = Client::new();
    let args = Args::parse();

    banner();

    let hn = if args.hn_flag {
        Some(fetch_top_ids_hn(&client).await?)
    } else {
        None
    };

    let rd = if args.rd_flag {
        Some(fetch_from_subreddit(&client, args.subreddit, args.limit).await?)
    } else {
        None
    };

    if let Some(hn) = hn {
        for id in hn.iter().take(args.limit) {
            match fetch_story_hn(&client, *id).await {
                Ok(story) => {
                    println!("{}", ORANGE);
                    println!("| Title: {:.<26} |", story.title.as_deref().unwrap_or("N/A"));
                    println!("| URL:   {:.<26} |", story.url.as_deref().unwrap_or("N/A"));
                    println!("| Score: {:.<26} |", story.score.map(|s| s.to_string()).unwrap_or_else(|| "N/A".to_string()));
                    println!("| {:-<30} ", "");
                    println!("{}", RESET);
                }
                Err(e) => {
                    eprintln!("{}Failed to fetch story {}: {}{}",RED, id, e, RESET);
                }
            }
        }
    }


    if let Some(rd) = rd {
        for post in rd.data.children.iter() {
            println!("{}", ORANGE);
            println!("| Title: {:.<26} |", post.data.title.as_deref().unwrap_or("N/A"));
            println!("| URL:   {:.<26} |", post.data.url.as_deref().unwrap_or("N/A"));
            println!("| UpVotes: {:.<26} |", post.data.score);
            println!("| {:-<30} ", "");
            println!("{}", RESET);
        }
    }


    
    Ok(())

}

fn banner() {


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






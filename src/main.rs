use std::io::{Write, stdin, stdout};
use std::{process};
use reqwest::Client;
use clap::{Parser};
use crate::fetch::hn::{fetch_top_ids_hn, fetch_story_hn};
use crate::fetch::rd::{fetch_from_subreddit};
use crate::fetch::ai::{fetch_ai_response};

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
                    process::exit(1);    
                }
                Err(e) => {
                    eprintln!("{}Failed to fetch story {}: {}{}",RED, id, e, RESET);
                    process::exit(1);    
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
        process::exit(1);    
    }

    let err = format!("{}Try Again{}",RED, RESET);
    
    loop {
        
        let mut line = String::new();

        print!("{}► {}", ORANGE, RESET);
        stdout().flush()?;

        stdin()
            .read_line(&mut line)
            .expect(&err);
        let line_trim = line.trim();

        if line_trim == "/quit" {
            println!("{}\n{} Thanks for using Trendy! Goodbye!  {}\n", CLEAR, ORANGE, RESET);
            break;
        }
        else if line_trim == "/clear" {
            println!("{}", CLEAR);
        }
        else if line_trim == "/rd" {
            let mut subreddit = String::new();
            let mut limit = String::new();
            let rd_flag: bool = true;

            print!("{}[/r/]► {}", ORANGE, RESET);
            stdout().flush()?;

            stdin()
                .read_line(&mut subreddit)
                .expect(&err);
            
            print!("{}[limit]► {}", ORANGE, RESET);
            stdout().flush()?;

            stdin()
                .read_line(&mut limit)
                .expect(&err);
            let limit_usize: usize = limit
                .trim()
                .parse()
                .expect(&err);

            let rd = if rd_flag {
                Some(fetch_from_subreddit(&client, subreddit, limit_usize).await?)
            } else {
                None
            };
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

        }
        else if line_trim == "/hn" {
            let mut limit = String::new();
            let hn_flag: bool = true;

            print!("{}[limit]► {}", ORANGE, RESET);
            stdout().flush()?;

            stdin()
                .read_line(&mut limit)
                .expect(&err);
            let limit_usize: usize = limit
                .trim()
                .parse()
                .expect(&err);

            let hn = if hn_flag {
                Some(fetch_top_ids_hn(&client).await?)
            } else {
                None
            };
            if let Some(hn) = hn {
                for id in hn.iter().take(limit_usize) {
                    match fetch_story_hn(&client, *id).await {
                        Ok(story) => {
                            println!("{}", ORANGE);
                            println!("| Title: {:.<26} |", story.title.as_deref().unwrap_or("N/A"));
                            println!("| URL:   {:.<26} |", story.url.as_deref().unwrap_or("N/A"));
                            println!("| Score: {:.<26} |", story.score.map(|s| s.to_string()).unwrap_or_else(|| "N/A".to_string()));
                            println!("| {:-<30} ", "");
                            println!("{}", RESET);
                            process::exit(1);    
                        }
                        Err(e) => {
                            eprintln!("{}Failed to fetch story {}: {}{}",RED, id, e, RESET);
                        }
                    }
                }
            }
        }
        else if line_trim == "/help" {
            println!(" ");
            println!("{}Available Commands for TrendyCLI REPL mode:{}", ORANGE, RESET);
            println!("{}  /help     - Show this help message{}", ORANGE, RESET);
            println!("{}  /clear    - Clear the screen{}", ORANGE, RESET);
            println!("{}  /rd       - Fetch posts from a subreddit{}", ORANGE, RESET);
            println!("{}  /hn       - Fetch top stories from Hacker News{}", ORANGE, RESET);
            println!("{}  /quit     - Exit the program{}", ORANGE, RESET);
            println!(" ");
        }
        else {
            let model: String = String::from("moonshotai/kimi-k2.5");
            match fetch_ai_response(&client, model, line_trim.to_string()).await {
                Ok(result) => {
                    println!(" ");
                    println!("{}[AI]► {}{}", ORANGE, result, RESET);
                    println!(" ");
                }
                Err(e) => {
                    eprintln!("{}Failed to fetch AI response: {}{}",RED, e, RESET);
                }
            }
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






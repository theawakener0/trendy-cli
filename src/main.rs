use std::io::{Write, stdin, stdout};
use std::{process};
use reqwest::Client;
use clap::{Parser};
use crate::fetch::hn::{fetch_top_ids_hn, fetch_story_hn};
use crate::fetch::rd::{fetch_from_subreddit};
use crate::fetch::ai::{fetch_ai_response_stream};

pub mod fetch; 


const RESET: &str = "\x1b[0m";
const CLEAR: &str = "\x1b[2J\x1b[H";
const ORANGE: &str = "\x1b[38;2;255;165;0m";
const RED: &str = "\x1b[0;31m";
const CLEAR_LINE: &str = "\r\x1b[K";


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 10)]
    limit: usize,
    
    #[arg(short, long, default_value = "rust")]
    subreddit: String,

    #[arg(short='n', long="hn")]
    hn_flag: bool,

    #[arg(short='r', long="rd")]
    rd_flag: bool,


}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

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
            
            let first_token = std::sync::Arc::new(std::sync::Mutex::new(false));
            let first_token_clone = first_token.clone();
            let first_token_for_callback = first_token.clone();
            
            let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let frame_idx = std::sync::Arc::new(std::sync::Mutex::new(0usize));
            let frame_idx_clone = frame_idx.clone();
            
            let spin_handle = std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(80));
                    let ft = first_token_clone.lock().unwrap();
                    if *ft {
                        break;
                    }
                    drop(ft);
                    let mut idx = frame_idx_clone.lock().unwrap();
                    let frame = spinner_frames[*idx % spinner_frames.len()];
                    *idx += 1;
                    drop(idx);
                    print!("\r{}{} Thinking....{}", ORANGE, frame, RESET);
                    stdout().flush().unwrap();
                }
            });
            
            print!("\r{}", CLEAR_LINE);
            print!("\r{}AI{}► ",ORANGE, ORANGE);
            stdout().flush().unwrap();
            
            match fetch_ai_response_stream(&client, model, line_trim.to_string(), move |token| {
                let mut ft = first_token_for_callback.lock().unwrap();
                if !*ft {
                    *ft = true;
                    drop(ft);
                    print!("\r{}{}{}", CLEAR_LINE, ORANGE, token);
                    stdout().flush().unwrap();
                } else {
                    print!("{}{}", ORANGE, token);
                    stdout().flush().unwrap();
                }
            }).await {
                Ok(_) => {
                    println!("{}", RESET);
                    println!(" ");
                }
                Err(e) => {
                    eprintln!("{}Failed to fetch AI response: {}{}",RED, e, RESET);
                }
            }
            
            let _ = spin_handle.join();
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






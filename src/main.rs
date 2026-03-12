use crate::config::get_api_key;
use crate::fetch::ai::{fetch_ai_models, fetch_ai_response_stream};
use crate::fetch::hn::{fetch_story_hn, fetch_top_ids_hn};
use crate::fetch::rd::fetch_from_subreddit;
use clap::Parser;
use reqwest::Client;
use std::error::Error;
use std::io::{self, stdin, stdout, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub mod config;
pub mod fetch;

type AppError = Box<dyn Error + Send + Sync>;

const RESET: &str = "\x1b[0m";
const CLEAR: &str = "\x1b[2J\x1b[H";
const ORANGE: &str = "\x1b[38;2;255;165;0m";
const RED: &str = "\x1b[0;31m";
const CLEAR_LINE: &str = "\r\x1b[K";
const DEFAULT_MODEL: &str = "moonshotai/kimi-k2.5";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    api_key: Option<String>,

    #[arg(short, long, default_value_t = 10)]
    limit: usize,

    #[arg(short, long, default_value = "rust")]
    subreddit: String,

    #[arg(short = 'n', long = "hn")]
    hn_flag: bool,

    #[arg(short = 'r', long = "rd")]
    rd_flag: bool,
}

enum RenderEvent {
    Token(String),
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenv::dotenv().ok();

    let client = Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(60))
        .user_agent("trendy-cli")
        .build()?;
    let args = Args::parse();

    let api_key = get_api_key(args.api_key);

    banner();

    let mut ran_command = false;

    if args.hn_flag {
        render_hn(&client, args.limit).await?;
        ran_command = true;
    }

    if args.rd_flag {
        render_rd(&client, args.subreddit.trim(), args.limit).await?;
        ran_command = true;
    }

    if ran_command {
        return Ok(());
    }

    repl(&client, api_key).await
}

async fn repl(client: &Client, api_key: Option<String>) -> Result<(), AppError> {
    let mut model = "".to_string();
    loop {
        let line = read_input("► ")?;

        if line.is_empty() {
            continue;
        }

        match line.as_str() {
            "/quit" => {
                println!(
                    "{}\n{} Thanks for using Trendy! Goodbye! {}\n",
                    CLEAR, ORANGE, RESET
                );
                break;
            }
            "/clear" => println!("{}", CLEAR),
            "/rd" => {
                let subreddit = read_input("[/r/]► ")?;
                let limit = read_limit()?;

                if let Err(err) = render_rd(client, subreddit.trim(), limit).await {
                    eprintln!("{}Failed to fetch subreddit posts: {}{}", RED, err, RESET);
                }
            }
            "/hn" => {
                let limit = read_limit()?;

                if let Err(err) = render_hn(client, limit).await {
                    eprintln!(
                        "{}Failed to fetch Hacker News stories: {}{}",
                        RED, err, RESET
                    );
                }
            }
            "/model" => {
                model = read_input("[Enter model name]► ")?;
            }
            "/models" => {
                if let Err(err) = render_get_ai_models(client).await {
                    eprintln!("{}Failed to fetch AI models: {}{}", RED, err, RESET);
                }
            }
            "/help" => print_help(),
            prompt => {
                if let Err(err) =
                    stream_ai_reply(client, api_key.clone(), model.as_mut_str(), prompt).await
                {
                    eprintln!("{}Failed to fetch AI response: {}{}", RED, err, RESET);
                }
            }
        }
    }

    Ok(())
}

fn read_input(prompt: &str) -> io::Result<String> {
    let mut line = String::new();

    print!("{}{}{}", ORANGE, prompt, RESET);
    stdout().flush()?;
    stdin().read_line(&mut line)?;

    Ok(line.trim().to_string())
}

fn read_limit() -> io::Result<usize> {
    loop {
        let raw = read_input("[limit]► ")?;

        match raw.parse::<usize>() {
            Ok(limit) if limit > 0 => return Ok(limit),
            _ => eprintln!("{}Enter a positive number.{}", RED, RESET),
        }
    }
}

async fn render_hn(client: &Client, limit: usize) -> Result<(), AppError> {
    let ids = fetch_top_ids_hn(client).await?;

    for id in ids.iter().take(limit) {
        match fetch_story_hn(client, *id).await {
            Ok(story) => print_hn_story(&story),
            Err(err) => eprintln!("{}Failed to fetch story {}: {}{}", RED, id, err, RESET),
        }
    }

    Ok(())
}

async fn render_rd(client: &Client, subreddit: &str, limit: usize) -> Result<(), AppError> {
    let rd = fetch_from_subreddit(client, subreddit.to_string(), limit).await?;

    for post in &rd.data.children {
        print_reddit_post(post);
    }

    Ok(())
}

async fn render_get_ai_models(client: &Client) -> Result<(), AppError> {
    let models = fetch_ai_models(client).await?;

    for model in models.data {
        print_models(&model);
    }

    Ok(())
}

async fn stream_ai_reply(
    client: &Client,
    api_key: Option<String>,
    mut model: &str,
    prompt: &str,
) -> Result<(), AppError> {
    let (tx, rx) = mpsc::channel();
    let render_handle = thread::spawn(move || render_ai_stream(rx));
    if model.is_empty() {
        model = DEFAULT_MODEL;
    }
    let result = fetch_ai_response_stream(
        client,
        api_key,
        model.to_string(),
        prompt.to_string(),
        move |token| {
            let _ = tx.send(RenderEvent::Token(token));
        },
    )
    .await;

    let rendered = render_handle.join().unwrap_or(false);

    if rendered {
        println!("{}", RESET);
        println!();
    }

    result
}

fn render_ai_stream(rx: mpsc::Receiver<RenderEvent>) -> bool {
    let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let mut spinner_idx = 0usize;
    let mut rendered = false;

    loop {
        match rx.recv_timeout(Duration::from_millis(80)) {
            Ok(RenderEvent::Token(token)) => {
                let mut chunk = token;
                while let Ok(RenderEvent::Token(next)) = rx.try_recv() {
                    chunk.push_str(&next);
                }

                if !rendered {
                    print!("\r\x1b[2K{}{}AI► {}{}", CLEAR_LINE, ORANGE, chunk, RESET);
                    rendered = true;
                } else {
                    print!("{}{}{}", ORANGE, chunk, RESET);
                }

                let _ = stdout().flush();
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if !rendered {
                    let frame = spinner_frames[spinner_idx % spinner_frames.len()];
                    spinner_idx += 1;
                    print!("\r{}{}{} Thinking...{}", CLEAR_LINE, ORANGE, frame, RESET);
                    let _ = stdout().flush();
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                if !rendered {
                    print!("\r{}", CLEAR_LINE);
                    let _ = stdout().flush();
                }
                break;
            }
        }
    }

    rendered
}

fn print_hn_story(story: &crate::fetch::hn::HNStory) {
    println!("{}", ORANGE);
    println!(
        "| Title: {:.<26} |",
        story.title.as_deref().unwrap_or("N/A")
    );
    println!("| URL:   {:.<26} |", story.url.as_deref().unwrap_or("N/A"));
    println!(
        "| Score: {:.<26} |",
        story
            .score
            .map(|score| score.to_string())
            .unwrap_or_else(|| "N/A".to_string())
    );
    println!("| {:-<30} ", "");
    println!("{}", RESET);
}

fn print_reddit_post(post: &crate::fetch::rd::RedditPost) {
    println!("{}", ORANGE);
    println!(
        "| Title: {:.<26} |",
        post.data.title.as_deref().unwrap_or("N/A")
    );
    println!(
        "| URL:   {:.<26} |",
        post.data.url.as_deref().unwrap_or("N/A")
    );
    println!("| UpVotes: {:.<23} |", post.data.score);
    println!("| {:-<30} ", "");
    println!("{}", RESET);
}

fn print_models(model: &crate::fetch::ai::ModelsData) {
    println!("{}ID:{} {}", ORANGE, RESET, model.id);
    if let Some(name) = &model.name {
        println!("{}Name:{} {}", ORANGE, RESET, name);
    }
    if let Some(desc) = &model.description {
        let short_desc = if desc.len() > 100 {
            format!("{}...", &desc[..100])
        } else {
            desc.clone()
        };
        println!("{}Desc:{} {}", ORANGE, RESET, short_desc);
    }
    println!();
}

fn print_help() {
    println!();
    println!(
        "{}Available Commands for TrendyCLI REPL mode:{}",
        ORANGE, RESET
    );
    println!("{}  /help     - Show this help message{}", ORANGE, RESET);
    println!("{}  /clear    - Clear the screen{}", ORANGE, RESET);
    println!(
        "{}  /rd       - Fetch posts from a subreddit{}",
        ORANGE, RESET
    );
    println!(
        "{}  /hn       - Fetch top stories from Hacker News{}",
        ORANGE, RESET
    );
    println!("{}  /model     - Change the DEFAULT_MODEL{}", ORANGE, RESET);
    println!("{}  /models    - List available AI models{}", ORANGE, RESET);
    println!("{}  /quit     - Exit the program{}", ORANGE, RESET);
    println!();
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
    println!("{} {} {}", ORANGE, banner, RESET);
}

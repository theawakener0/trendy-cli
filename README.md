# TrendyCLI

A simple Rust CLI tool for fetching data from reddit, and hacker news with AI chat.

## Features

- **Reddit**: Fetch posts from any subreddit
- **Hacker News**: Get top stories from Hacker News
- **AI Chat**: Interact with AI models via HackClubAI

## Installation

```bash
cargo install trendy-cli
```

Or build from source:

```bash
# COPY AND PASTE (if you have rust installed).

git clone https://github.com/theawakener0/trendy-cli.git
cd trendy-cli

cargo build --release

```

## Usage

### Command Line Options

```bash
trendy-cli --help
```

```
Options:
  -a, --api-key <API_KEY>      HackClubAI API key
  -l, --limit <LIMIT>          Number of items to fetch [default: 10]
  -s, --subreddit <SUBREDDIT>  Subreddit to fetch [default: rust]
  -n, --hn                     Fetch Hacker News top stories
  -r, --rd                     Fetch Reddit posts
  -h, --help                   Print help
```

### Fetch Reddit Posts

```bash
trendy-cli -r -l 5 -s programming
```

### Fetch Hacker News Stories

```bash
trendy-cli -n -l 10
```

### Interactive Mode (REPL)

Run without flags to enter interactive mode:

```bash
trendy-cli
```

#### REPL Commands

| Command | Description |
|---------|-------------|
| `/help` | Show available commands |
| `/clear` | Clear the screen |
| `/rd` | Fetch posts from a subreddit |
| `/hn` | Fetch top Hacker News stories |
| `/model` | Change the AI model |
| `/models` | List available AI models |
| `/quit` | Exit the program |

## Configuration

### API Key

Set your HackClubAI API key via:
- Command line: `-a` or `--api-key` flag
- JSON config: `~/.config/trendy-cli/config.json`
- Environment variable: `HACKCLUB_API_KEY`
- `.env` file in the project directory

### Default Model

The default AI model is `moonshotai/kimi-k2.5`. Change it using the `/model` and `/models` to view the available models command in REPL mode.

## License

[MIT License](LICENSE).

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use core::fmt;
use dialoguer::{theme::ColorfulTheme, Select};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// GIthub access token
    #[arg(short, long)]
    token: String,
}

#[derive(Deserialize, Debug)]
struct Issue {
    html_url: String,
    number: usize,
    title: String,
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(Issue {}: {})", self.number, self.title)
    }
}

#[derive(Serialize, Deserialize)]
struct Config {
    github_access_token: String,
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            github_access_token: String::from(""),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = reqwest::Client::new();
    let mut cfg: Config = confy::load("issue-tracker", None).unwrap_or_else(|err| {
        eprintln!("{}: {}", "Error".red().bold(), err);
        std::process::exit(1);
    });
    let args = Args::parse();

    if !args.token.is_empty() {
        let new_config = Config {
            github_access_token: args.token,
        };

        confy::store("issue-tracker", None, &new_config).unwrap_or_else(|err| {
            eprintln!("{}: {}", "Error".red().bold(), err);
            std::process::exit(1);
        });

        cfg.github_access_token = new_config.github_access_token;
    }

    if cfg.github_access_token.is_empty() {
        eprintln!(
            "{}: No Github access token set. Please set one with the --token (-t) flag.",
            "Error".red().bold()
        );
        std::process::exit(1);
    }

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_style(
        ProgressStyle::with_template("{spinner} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "-"]),
    );
    pb.set_message("Fetching issues...");

    let res = client
        .get("https://api.github.com/issues")
        .header(
            AUTHORIZATION,
            format!("Bearer {}", cfg.github_access_token),
        )
        .header(ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header(USER_AGENT, "BrookJeynes")
        .send()
        .await
        .unwrap_or_else(|err| {
            eprintln!("{}: {}", "Error".red().bold(), err);
            std::process::exit(1);
        })
        .json::<Vec<Issue>>()
        .await
        .unwrap_or_else(|err| {
            eprintln!("{}: {}", "Error".red().bold(), err);
            std::process::exit(1);
        });

    pb.finish_with_message("Found issues! Press <C-c> to quit");

    loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select an issue:")
            .default(0)
            .items(&res[..])
            .interact()
            .unwrap_or_else(|err| {
                eprintln!("{}: {}", "Error".red().bold(), err);
                std::process::exit(1);
            });

        webbrowser::open(res[selection].html_url.as_str()).unwrap_or_else(|err| {
            eprintln!("{}: {}", "Error".red().bold(), err);
            std::process::exit(1);
        });
    }
}

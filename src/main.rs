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
    /// Github access token
    #[arg(short, long)]
    token: Option<String>,

    /// Github user name
    #[arg(short, long)]
    user_name: Option<String>,

    /// Print the config file path
    #[clap(short, long, action)]
    file_path: bool,
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
    user_name: String,
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            github_access_token: String::from(""),
            user_name: String::from(""),
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

    if args.file_path {
        eprintln!("{:?}", confy::get_configuration_file_path("issue-tracker", None).unwrap());
        std::process::exit(1);
    }


    let new_config = Config {
        github_access_token: args.token.unwrap_or(String::new()),
        user_name: args.user_name.unwrap_or(String::new()),
    };

    if !new_config.github_access_token.is_empty()
        && new_config.github_access_token != cfg.github_access_token
    {
        cfg.github_access_token = new_config.github_access_token;
    }

    if !new_config.user_name.is_empty() && new_config.user_name != cfg.user_name {
        cfg.user_name = new_config.user_name;
    }

    confy::store("issue-tracker", None, &cfg).unwrap_or_else(|err| {
        eprintln!("{}: {}", "Error".red().bold(), err);
        std::process::exit(1);
    });

    if cfg.github_access_token.is_empty() {
        eprintln!(
            "{}: No Github access token set. Please set one with the --token (-t) flag.",
            "Error".red().bold()
        );
        std::process::exit(1);
    }

    if cfg.user_name.is_empty() {
        eprintln!(
            "{}: No Github user name is set. Please set one with the --user-name (-u) flag.",
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
        .header(AUTHORIZATION, format!("Bearer {}", cfg.github_access_token))
        .header(ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header(USER_AGENT, cfg.user_name)
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

pub mod models;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Select};
use indicatif::{ProgressBar, ProgressStyle};
use models::{args::Args, config::Config, issue::Issue};
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use std::time::Duration;

fn load_new_config(config: &mut Config, new_config: Config) {
    if !new_config.github_access_token.is_empty()
        && new_config.github_access_token != config.github_access_token
    {
        config.github_access_token = new_config.github_access_token;
    }

    if !new_config.user_name.is_empty() && new_config.user_name != config.user_name {
        config.user_name = new_config.user_name;
    }

    confy::store("issue-tracker", None, &config).unwrap_or_else(|err| {
        eprintln!("{}: {}", "Error".red().bold(), err);
        std::process::exit(1);
    });
}

fn create_spinner() -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.enable_steady_tick(Duration::from_millis(120));
    spinner.set_style(
        ProgressStyle::with_template("{spinner} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "-"]),
    );
    spinner.set_message("Fetching issues...");

    spinner
}

fn check_empty_values(config: &Config) {
    if config.github_access_token.is_empty() {
        eprintln!(
            "{}: No Github access token set. Please set one with the --token (-t) flag.",
            "Error".red().bold()
        );
        std::process::exit(1);
    }

    if config.user_name.is_empty() {
        eprintln!(
            "{}: No Github user name is set. Please set one with the --user-name (-u) flag.",
            "Error".red().bold()
        );
        std::process::exit(1);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = reqwest::Client::new();
    let mut config: Config = confy::load("issue-tracker", None).unwrap_or_else(|err| {
        eprintln!("{}: {}", "Error".red().bold(), err);
        std::process::exit(1);
    });
    let args = Args::parse();

    if args.file_path {
        eprintln!(
            "{:?}",
            confy::get_configuration_file_path("issue-tracker", None).unwrap()
        );
        std::process::exit(1);
    }

    let new_config = Config {
        github_access_token: args.token.unwrap_or(String::new()),
        user_name: args.user_name.unwrap_or(String::new()),
    };

    load_new_config(&mut config, new_config);

    check_empty_values(&config);

    let spinner = create_spinner();

    let res = client
        .get("https://api.github.com/issues")
        .header(
            AUTHORIZATION,
            format!("Bearer {}", config.github_access_token),
        )
        .header(ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header(USER_AGENT, config.user_name)
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

    spinner.finish_with_message("Found issues! Press <C-c> to quit");

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

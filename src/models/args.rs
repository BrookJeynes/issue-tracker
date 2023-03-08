use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Github access token
    #[arg(short, long)]
    pub token: Option<String>,

    /// Github user name
    #[arg(short, long)]
    pub user_name: Option<String>,

    /// Print the config file path
    #[clap(short, long, action)]
    pub file_path: bool,
}

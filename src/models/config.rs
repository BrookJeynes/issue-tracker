use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub github_access_token: String,
    pub user_name: String,
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            github_access_token: String::from(""),
            user_name: String::from(""),
        }
    }
}

use core::fmt;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Issue {
    pub html_url: String,
    pub number: usize,
    pub title: String,
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(Issue {}: {})", self.number, self.title)
    }
}

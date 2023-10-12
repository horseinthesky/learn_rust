use std::fmt;

use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum UpstreamStatus {
    Running,
    Starting,
    Shutdown,
}

impl fmt::Display for UpstreamStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UpstreamStatus::Running => write!(f, "running"),
            UpstreamStatus::Starting => write!(f, "starting"),
            UpstreamStatus::Shutdown => write!(f, "shutdown"),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Info {
    pub node: String,
    pub upstream: String,
    pub status: UpstreamStatus,
}

pub fn federify(host: &str) -> String {
    format!("http://{}:15672/api/federation-links", host)
}

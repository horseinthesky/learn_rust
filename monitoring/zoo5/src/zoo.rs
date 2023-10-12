use std::collections::HashMap;
use std::error::Error;

use tokio::io::BufReader;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::Duration;
use tracing::debug;

use crate::juggler::{Event, Status};

const ZOO_CMD: &str = "mntr";
const TIMEOUT: u64 = 2000;

#[derive(Debug)]
enum TimeoutError {
    Open(String),
    Read(String),
}

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeoutError::Open(host) => {
                write!(f, "timed out opening connection to {host}")
            }
            TimeoutError::Read(host) => {
                write!(f, "timed out reading data from {host}")
            }
        }
    }
}

impl Error for TimeoutError {}

#[derive(Debug)]
enum CollectError {
    Connect(String, Box<dyn Error>),
    Write(String, Box<dyn Error>),
    Read(String, Box<dyn Error>),
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connect(host, error) => {
                write!(f, "failed to connect to {host}: {error}")
            }
            Self::Write(host, error) => {
                write!(f, "failed to write command to {host}: {error}")
            }
            Self::Read(host, error) => {
                write!(f, "failed to read data from {host}: {error}")
            }
        }
    }
}

impl Error for CollectError {}

pub async fn collect(host: &str) -> Result<(&str, String), Box<dyn Error>> {
    debug!("connecting to {host}...");

    let mut conn = match tokio::time::timeout(
        Duration::from_millis(TIMEOUT),
        TcpStream::connect(format!("{host}:2181")),
    )
    .await
    {
        Err(_elapsed) => return Err(Box::new(TimeoutError::Open(host.to_string()))),
        Ok(Err(e)) => {
            return Err(Box::new(CollectError::Connect(
                host.to_string(),
                Box::new(e),
            )))
        }
        Ok(Ok(stream)) => stream,
    };

    match conn.write_all(ZOO_CMD.as_bytes()).await {
        Err(e) => return Err(Box::new(CollectError::Write(host.to_string(), Box::new(e)))),
        Ok(_) => (),
    };

    let mut buf = String::new();
    let mut reader = BufReader::new(conn);

    match tokio::time::timeout(Duration::from_millis(100), reader.read_to_string(&mut buf)).await {
        Err(_elapsed) => Err(Box::new(TimeoutError::Read(host.to_string()))),
        Ok(Err(e)) => Err(Box::new(CollectError::Read(host.to_string(), Box::new(e)))),
        Ok(Ok(_)) => {
            debug!("got data from {host}");
            Ok((host, buf))
        }
    }
}

pub fn compute(host: &str, info: String, expected_followers: usize) -> Event {
    let mut zoo_info = HashMap::new();

    for line in info.lines() {
        let key_value: Vec<_> = line.split("\t").collect();
        let k = key_value[0];
        let v = key_value[1];

        if k == "zk_server_state" {
            zoo_info.insert("state", v);
        }

        if k == "zk_synced_followers" {
            zoo_info.insert("followers", v);
            break;
        }
    }

    let description;
    let status;

    if zoo_info.get("state").unwrap() == &"follower" {
        description = String::from("follower");
        status = Status::OK;
    } else {
        let followers = zoo_info.get("followers").unwrap().parse::<usize>().unwrap();

        description = format!("leader. followers: {followers}/{expected_followers}");

        status = if followers == expected_followers {
            Status::OK
        } else {
            Status::WARN
        };
    };

    Event {
        host: format!("{host}-test"),
        service: "state",
        instance: "",
        status,
        description,
        tags: vec!["zoo", "k8s", "monitoring"],
    }
}

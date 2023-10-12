use std::collections::HashMap;

use tokio::io::BufReader;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::Duration;
use tracing::debug;

use crate::juggler::{Event, Status};

const ZOO_CMD: &str = "mntr";
const TIMEOUT: u64 = 200;

pub async fn collect(host: &str) -> Result<(&str, String), Box<dyn std::error::Error>> {
    debug!("connecting to {host}...");

    let mut conn = tokio::time::timeout(
        Duration::from_millis(TIMEOUT),
        TcpStream::connect(format!("{host}:2181")),
    )
    .await??;

    conn.write_all(ZOO_CMD.as_bytes()).await?;

    let mut buf = String::new();
    let mut reader = BufReader::new(conn);

    tokio::time::timeout(Duration::from_millis(100), reader.read_to_string(&mut buf)).await?.ok();

    debug!("got data from {host}");
    Ok((host, buf))
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

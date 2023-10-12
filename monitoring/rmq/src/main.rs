mod juggler;
mod rmq;

use reqwest::{header::CONTENT_TYPE, ClientBuilder};
use serde::Deserialize;

use juggler::{jugglerify, Event, Payload, Status};
use rmq::{federify, Info, UpstreamStatus};

const TIMEOUT: u64 = 3;

#[derive(Deserialize, Debug)]
struct Config {
    rmq_hosts: String,
    rmq_login: String,
    rmq_password: String,
    juggler_url: String,
}

fn compute(info: Vec<Info>) -> Event {
    let status = if info.iter().all(|i| i.status == UpstreamStatus::Running) {
        Status::OK
    } else {
        Status::WARN
    };

    let node = info[0].node.split("@").last().unwrap();

    let mut description = format!("Federation {} upstreams status:\n", node);
    for i in &info {
        let upstream_name = i.upstream.split(".").collect::<Vec<&str>>()[0];
        description = format!("{}\n{}: {}", description, upstream_name, i.status);
    }

    Event {
        host: format!("{}-test", node),
        service: String::from("federation"),
        instance: String::from(""),
        status,
        description,
        tags: vec![],
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = envy::from_env::<Config>()
        .expect("RMQ_HOSTS and RMQ_LOGIN and RMQ_PASSWORD and JUGGLER_URL env vars must be set");

    let client = ClientBuilder::new()
        .timeout(tokio::time::Duration::from_secs(TIMEOUT))
        .build()
        .unwrap();

    let hosts = config.rmq_hosts.split(",");

    let bodies = futures::future::join_all(hosts.into_iter().map(|host| {
        let url: String = federify(host);

        let client = &client;
        let login = &config.rmq_login;
        let password = &config.rmq_password;

        async move {
            client
                .get(url)
                .basic_auth(login, Some(password))
                .send()
                .await?
                .json()
                .await
        }
    }))
    .await;

    let events = bodies
        .into_iter()
        .filter(|b| match b {
            Err(e) => {
                log::warn!("failed to reach RMQ: {}", e);
                false
            }
            _ => true,
        })
        .map(|b| compute(b.unwrap()))
        .collect();

    // println!("{:?}", events);

    let payload = Payload {
        source: String::from("rmq"),
        events,
    };

    let res = client
        .post(jugglerify(config.juggler_url))
        .header(CONTENT_TYPE, "application/json")
        .json(&payload)
        .send()
        .await;

    let response = match res {
        Err(e) => {
            log::error!("failed to reach Juggler: {:?}", e);
            return;
        }
        Ok(response) => response,
    };

    let reply: Result<serde_json::Value, reqwest::Error> = response.json().await;

    match reply {
        Ok(o) => log::info!("rmq monitoring completed successfully. Reply: {:#?}", o),
        Err(e) => log::error!("failed to parse Juggler response: {:?}", e),
    }
}

use reqwest::{header::CONTENT_TYPE, ClientBuilder};
use serde::Deserialize;
use std::process;
use tracing::{error, info, warn};

mod juggler;
use juggler::{jugglerify, Event, Payload};

mod zoo;
use zoo::{collect, compute, ZooError};

const TIMEOUT: u64 = 3;

#[derive(Deserialize, Debug)]
struct Config {
    zoo_hosts: String,
    juggler_url: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config =
        envy::from_env::<Config>().expect("ZOO_HOSTS and JUGGLER_URL env vars must be set");

    let hosts = config.zoo_hosts.split(",");

    let responses: Vec<Result<(&str, String), ZooError>> =
        futures::future::join_all(hosts.map(|host| async move {
            collect(host)
                .await
                .map_err(|err| ZooError(format!("{host}: {err}")))
        }))
        .await;

    let expected_followers = config.zoo_hosts.split(",").count() - 1;

    let events: Vec<Event> = responses
        .into_iter()
        .filter(|r| match r {
            Err(e) => {
                warn!("{e}");
                false
            }
            _ => true,
        })
        .map(|r| {
            let (host, data) = r.unwrap();
            let hostname = host.split(".").next().unwrap();

            compute(hostname, data, expected_followers)
        })
        .collect();

    if events.is_empty() {
        warn!("zoo monitoring has no events to send");
        process::exit(1)
    }

    let payload = Payload {
        source: String::from("zoo"),
        events,
    };

    let client = ClientBuilder::new()
        .timeout(tokio::time::Duration::from_secs(TIMEOUT))
        .build()
        .unwrap();

    let res = client
        .post(jugglerify(config.juggler_url))
        .header(CONTENT_TYPE, "application/json")
        .json(&payload)
        .send()
        .await;

    let response = match res {
        Err(e) => {
            error!("failed to reach Juggler: {:?}", e);
            process::exit(1)
        }
        Ok(response) => response,
    };

    let reply: Result<serde_json::Value, reqwest::Error> = response.json().await;
    if reply.is_err() {
        error!("failed to parse Juggler response: {:?}", reply.err());
        process::exit(1)
    }

    info!("zoo monitoring completed successfully.")
}

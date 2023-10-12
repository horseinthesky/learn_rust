use serde::Serialize;

#[allow(dead_code)]
#[derive(Serialize, Debug)]
pub enum Status {
    OK,
    WARN,
    CRIT,
}

#[derive(Serialize, Debug)]
pub struct Event {
    pub host: String,
    pub service: String,
    pub instance: String,
    pub status: Status,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Serialize)]
pub struct Payload {
    pub source: String,
    pub events: Vec<Event>,
}

pub fn jugglerify(fqdn: String) -> String {
     format!("{}/events", fqdn)
}

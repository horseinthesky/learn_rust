use serde::Serialize;

#[allow(dead_code)]
#[derive(Serialize, Debug)]
pub enum Status {
    OK,
    WARN,
    CRIT,
}

#[derive(Serialize, Debug)]
pub struct Event<'a> {
    pub host: String,
    pub service: &'a str,
    pub instance: &'a str,
    pub status: Status,
    pub description: String,
    pub tags: Vec<&'a str>,
}

#[derive(Serialize)]
pub struct Payload<'a> {
    pub source: String,
    pub events: Vec<Event<'a>>,
}

pub fn jugglerify(fqdn: String) -> String {
    format!("{}/events", fqdn)
}

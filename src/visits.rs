use crate::utils::*;
use log::info;
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    io::Write,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::RwLock;

/// A recorded visit to the server.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Visit {
    pub timestamp: u64, // The moment the request came. Seconds since unix epoch in UTC.
    pub handling_duration: u128, // Time it took to handle the request in microseconds.
    pub response_status_code: u16,
    pub method: String,
    pub url: String,
    pub user_agent: String,
    pub language: String,
}

/// Functionality for easily tracing a visit.
impl Visit {
    pub fn start(request: &Request) -> OngoingVisit {
        OngoingVisit {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards.")
                .as_secs(),
            start: Instant::now(),
            method: format!("{}", request.method),
            url: request.path.join("/"),
            user_agent: request.user_agent.clone(),
            language: request.language.clone(),
        }
    }
}
#[derive(Debug)]
pub struct OngoingVisit {
    timestamp: u64,
    start: Instant,
    method: String,
    url: String,
    user_agent: String,
    language: String,
}
impl OngoingVisit {
    pub fn end(self, response: &Response) -> Visit {
        Visit {
            timestamp: self.timestamp,
            handling_duration: (Instant::now() - self.start).as_micros(),
            response_status_code: response.status().as_u16(),
            method: self.method,
            url: self.url,
            user_agent: self.user_agent,
            language: self.language,
        }
    }
}

/// A log of visits that maintains some statistics.
///
/// It simply stores all visits in a vector. When that becomes too large, some
/// are dumped to disk.
pub struct VisitsLog {
    visits: Vec<Visit>,
}
impl VisitsLog {
    const NUM_BUFFER_MIN: usize = 100;
    const NUM_BUFFER_MAX: usize = 1000;

    pub fn new() -> Self {
        Self { visits: vec![] }
    }

    pub fn register(&mut self, visit: Visit) {
        info!("Registered visit: {:?}", visit);
        self.visits.push(visit);

        if self.visits.len() > Self::NUM_BUFFER_MAX {
            info!("Dumping visits to disk.");
            let rest = self
                .visits
                .split_off(Self::NUM_BUFFER_MAX - Self::NUM_BUFFER_MIN);
            let to_disk = std::mem::replace(&mut self.visits, rest);

            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open("visits.jsonl")
                .unwrap();
            for visit in to_disk {
                let json = serde_json::to_string(&visit).unwrap();
                file.write(json.as_bytes()).unwrap();
                file.write(&[10]).unwrap(); // '\n'
            }
        }
    }

    fn last_100(&self) -> &[Visit] {
        if self.visits.len() < 100 {
            &self.visits[..]
        } else {
            &self.visits[self.visits.len() - 100..]
        }
    }
}

pub async fn handle(db: &RwLock<VisitsLog>, request: &Request) -> Option<Response> {
    if request.path.starts_with(vec!["api", "visits"]) {
        let rest_of_path: Vec<String> = request.path.clone_except_first(2);
        if !request.is_admin {
            return Some(not_authenticated_page().await);
        }
        if request.method == Method::GET && rest_of_path == vec!["last"] {
            let json = serde_json::to_string(&db.read().await.last_100()).unwrap();
            return Some(Response::with_body(json.into()));
        }
    }

    None
}

use crate::utils::*;
use log::info;
use serde::{Deserialize, Serialize};
use std::{
    sync::RwLock,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

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

pub struct VisitsLog {
    visits: Vec<Visit>,
}
impl VisitsLog {
    pub fn new() -> Self {
        Self { visits: vec![] }
    }

    pub fn register(&mut self, visit: Visit) {
        info!("Registered visit: {:?}", visit);
        self.visits.push(visit);
    }

    pub fn list(&self) -> Vec<Visit> {
        self.visits.clone()
    }
}

pub fn handle(db: &RwLock<VisitsLog>, request: &Request) -> Option<Response> {
    if request.path.starts_with(vec!["api", "visits"]) {
        let rest_of_path: Vec<String> = request.path.clone_except_first(2);
        if !request.is_admin {
            return Some(not_authenticated_page());
        }
        if request.method == Method::GET && rest_of_path.is_empty() {
            let json = serde_json::to_string(&db.read().unwrap().list()).unwrap();
            return Some(Response::with_body(json.into()));
        }
    }

    None
}

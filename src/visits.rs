use std::{
    str::FromStr,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use log::info;
use tiny_http::{HeaderField, Request};

use crate::handlers::{Response, StatusCode};

/// A recorded visit to the server.
#[derive(Debug)]
pub struct Visit {
    pub timestamp: u64, // The moment the request came. Seconds since unix epoch in UTC.
    pub handling_duration: u128, // Time it took to handle the request in microseconds.
    pub response_status_code: StatusCode,
    pub method: String,
    pub url: String,
    pub user_agent: String,
    pub language: String,
}

/// Functionality for easily tracing a visit.
impl Visit {
    pub fn start(request: &Request) -> OngoingVisit {
        fn get_header_value(request: &Request, field: &str) -> String {
            request
                .headers()
                .iter()
                .filter(|header| header.field == HeaderField::from_str(field).unwrap())
                .next()
                .map(|header| header.value.clone().into())
                .unwrap_or("".into())
        }
        OngoingVisit {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards.")
                .as_secs(),
            start: Instant::now(),
            method: format!("{}", request.method()),
            url: request.url().into(),
            user_agent: get_header_value(&request, "User-Agent"),
            language: get_header_value(&request, "Accept-Language"),
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
            response_status_code: response.status_code,
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
}

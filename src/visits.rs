use crate::utils::*;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, DurationMicroSeconds, TimestampSeconds};
use std::{
    collections::VecDeque,
    fs::OpenOptions,
    io::Write,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

/// A recorded visit to the server.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Visit {
    #[serde_as(as = "TimestampSeconds")]
    pub timestamp: DateTime<Utc>,
    #[serde_as(as = "DurationMicroSeconds")]
    pub handling_duration: Duration,
    pub response_status: Result<u16, String>,
    pub method: String,
    pub url: String,
    pub user_agent: Option<String>,
    pub language: Option<String>,
}
impl Visit {
    pub fn for_request(req: &ServiceRequest) -> OngoingVisit {
        OngoingVisit {
            timestamp: Utc::now(),
            handling_start: Instant::now(),
            method: req.method().to_string(),
            url: req.path().to_owned(),
            user_agent: req.headers().get_utf8("user-agent"),
            language: req.headers().get_utf8("language"),
        }
    }
}
pub struct OngoingVisit {
    pub timestamp: DateTime<Utc>,
    pub handling_start: Instant,
    pub method: String,
    pub url: String,
    pub user_agent: Option<String>,
    pub language: Option<String>,
}
impl OngoingVisit {
    pub fn finish(self, res: &Result<ServiceResponse, actix_web::Error>) -> Visit {
        Visit {
            timestamp: self.timestamp,
            handling_duration: Instant::now() - self.handling_start,
            response_status: match res {
                Ok(res) => Ok(res.status().as_u16()),
                Err(err) => Err(format!("{:?}", err)),
            },
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
#[derive(Default, Clone)]
pub struct VisitsLog {
    /// The ground truth of all visits. It's simply filled up to `BUFFER_SIZE`
    /// and then flushed to disk.
    buffer: Arc<RwLock<Vec<Visit>>>,

    /// A deque containing the last `TAIL_SIZE` visits.
    tail: Arc<RwLock<VecDeque<Visit>>>,
}
impl VisitsLog {
    const BUFFER_SIZE: usize = 1000;
    const TAIL_SIZE: usize = 100;

    pub fn new() -> Self {
        Self {
            buffer: Default::default(),
            tail: Default::default(),
        }
    }

    pub async fn register(&self, visit: Visit) {
        info!("Registering visit: {:?}", visit);

        let mut buffer = self.buffer.write().await;
        buffer.push(visit.clone());
        if buffer.len() > Self::BUFFER_SIZE {
            self.flush().await
        }

        let mut tail = self.tail.write().await;
        tail.push_back(visit);
        if tail.len() > Self::TAIL_SIZE {
            tail.pop_front();
        }
    }

    async fn flush(&self) {
        info!("Flushing visits to disk.");
        let buffer = std::mem::replace(&mut *self.buffer.write().await, vec![]);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("visits.jsonl")
            .unwrap();
        for visit in buffer {
            let json = serde_json::to_string(&visit).unwrap();
            file.write(json.as_bytes()).unwrap();
            file.write(&[10]).unwrap(); // '\n'
        }
    }

    pub async fn get_tail(&self) -> Vec<Visit> {
        self.tail.read().await.clone().into_iter().rev().collect()
    }
}

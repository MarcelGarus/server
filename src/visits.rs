use crate::utils::*;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, DurationMicroSeconds, TimestampSeconds};
use std::{
    collections::{HashMap, VecDeque},
    iter::FromIterator,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::io::{self, AsyncWriteExt};
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
    pub referer: Option<String>,
}
impl Visit {
    pub fn for_request(req: &ServiceRequest) -> OngoingVisit {
        OngoingVisit {
            timestamp: Utc::now(),
            handling_start: Instant::now(),
            method: req.method().to_string(),
            url: req.path().to_owned(),
            user_agent: req.headers().get_utf8("user-agent"),
            language: req.headers().get_utf8("accept-language"),
            referer: req.headers().get_utf8("referer"),
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
    pub referer: Option<String>,
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
            referer: self.referer,
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

    tail: LimitedLog<Visit>,
    number_of_visits: Arc<RwLock<HashMap<UtcDate, u64>>>,
}
impl VisitsLog {
    const BUFFER_SIZE: usize = 100;

    pub async fn new() -> Self {
        Self {
            buffer: Default::default(),
            tail: Default::default(),
            number_of_visits: Default::default(),
        }
    }

    pub async fn register(&self, visit: Visit) {
        info!("Registering {:?}", visit);

        let buffer_size = {
            let mut buffer = self.buffer.write().await;
            buffer.push(visit.clone());
            buffer.len()
        };
        if buffer_size > Self::BUFFER_SIZE {
            self.flush().await.expect("Couldn't flush visits to disk.");
        }

        self.register_for_stats(visit).await;
    }

    pub async fn flush(&self) -> io::Result<()> {
        info!("Flushing visits to disk.");
        let mut buffer = self.buffer.write().await;
        let visits = buffer.clone();
        buffer.clear();

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("visits.jsonl")
            .await?;
        for visit in visits {
            let json = serde_json::to_string(&visit).unwrap();
            file.write_all(json.as_bytes()).await?;
            file.write_all(&[10]).await?; // '\n'
        }
        info!("Visits successfully flushed to disk.");
        Ok(())
    }

    async fn register_for_stats(&self, visit: Visit) {
        let date = UtcDate(visit.timestamp.date());
        self.tail.add(visit.clone()).await;

        let mut number_of_visits = self.number_of_visits.write().await;
        *number_of_visits.entry(date.clone()).or_insert(0) += 1;
        let a_month_ago = Utc::now()
            .date()
            .checked_sub_signed(chrono::Duration::days(30))
            .unwrap();
        number_of_visits.retain(|date, _| date.0 > a_month_ago);
    }

    pub async fn get_tail(&self) -> Vec<Visit> {
        self.tail.list().await
    }

    pub async fn get_number_of_visits_by_day(&self) -> HashMap<UtcDate, u64> {
        HashMap::from_iter(self.number_of_visits.read().await.clone())
    }
}

/// Log that keeps a maximum number of items.
#[derive(Clone)]
struct LimitedLog<T: Clone> {
    log: Arc<RwLock<VecDeque<T>>>,
}
impl<T: Clone> LimitedLog<T> {
    const LOG_SIZE: usize = 100;

    async fn add(&self, item: T) {
        let mut log = self.log.write().await;
        log.push_back(item);
        if log.len() > Self::LOG_SIZE {
            log.pop_front();
        }
    }
    async fn list(&self) -> Vec<T> {
        self.log.read().await.clone().into_iter().rev().collect()
    }
}
impl<T: Clone> Default for LimitedLog<T> {
    fn default() -> Self {
        Self {
            log: Default::default(),
        }
    }
}

use crate::utils::*;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, DurationMicroSeconds, TimestampSeconds};
use std::{
    collections::{HashMap, VecDeque},
    io::BufRead,
    iter::FromIterator,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};
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
    errors: LimitedLog<Visit>,
    number_of_visits: MonthlyDistributionCounter<()>,
    visited_urls: MonthlyDistributionCounter<String>,
    user_agents: MonthlyDistributionCounter<String>,
    languages: MonthlyDistributionCounter<String>,
    referers: MonthlyDistributionCounter<String>,
}
impl VisitsLog {
    const BUFFER_SIZE: usize = 100;

    pub async fn new() -> Self {
        let log = Self {
            buffer: Default::default(),
            tail: Default::default(),
            errors: Default::default(),
            number_of_visits: Default::default(),
            visited_urls: Default::default(),
            user_agents: Default::default(),
            languages: Default::default(),
            referers: Default::default(),
        };
        // We intentionally use synchronous File I/O here. This only happens on
        // start and on the small server, the file I/O is actually faster than
        // we can process the lines. That's why when the tokio frameworks tries
        // to send more and more data to us and fills our buffers, a "Resource
        // temporarily unavailable" error would occur.
        // info!("Reading existing visits.");
        // let file = std::fs::File::open("visits.jsonl").expect("Can't open visits.jsonl");
        // let visits = std::io::BufReader::new(file)
        //     .lines()
        //     .map(|line| line.expect("Couldn't read visits line.").trim().to_owned())
        //     .filter(|line| !line.is_empty())
        //     .map(|line| {
        //         serde_json::from_str(&line).expect(&format!("Invalid visit line: {}", line))
        //     });
        // for visit in visits {
        //     log.register_for_stats(visit).await;
        // }
        // info!("Done reading existing visits.");
        log
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
        // TODO: Make this more efficient
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

        if !matches!(visit.response_status, Ok(200 | 301)) {
            self.errors.add(visit.clone()).await;
        }

        self.number_of_visits.report_occurrence(&date, ()).await;

        self.visited_urls.report_occurrence(&date, visit.url).await;

        if let Some(user_agent) = visit.user_agent {
            self.user_agents.report_occurrence(&date, user_agent).await;
        }

        if let Some(language) = visit.language {
            self.languages.report_occurrence(&date, language).await;
        }

        if let Some(referer) = visit.referer {
            self.referers.report_occurrence(&date, referer).await;
        }
    }

    pub async fn get_tail(&self) -> Vec<Visit> {
        self.tail.list().await
    }

    pub async fn get_error_tail(&self) -> Vec<Visit> {
        self.errors.list().await
    }

    pub async fn get_number_of_visits_by_day(&self) -> HashMap<UtcDate, u64> {
        HashMap::from_iter(
            self.number_of_visits
                .list()
                .await
                .into_iter()
                .map(|(date, map)| (date, *map.get(&()).unwrap_or(&0))),
        )
    }

    pub async fn get_urls_by_day(&self) -> HashMap<UtcDate, HashMap<String, u64>> {
        self.visited_urls.list().await
    }

    pub async fn get_user_agents_by_day(&self) -> HashMap<UtcDate, HashMap<String, u64>> {
        self.user_agents.list().await
    }

    pub async fn get_languages_by_day(&self) -> HashMap<UtcDate, HashMap<String, u64>> {
        self.languages.list().await
    }

    pub async fn get_referers_by_day(&self) -> HashMap<UtcDate, HashMap<String, u64>> {
        self.referers.list().await
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

/// Counts the occurrence of some thing in the last 30 days.
#[derive(Clone, Default)]
struct MonthlyDistributionCounter<T: Clone + Eq + core::hash::Hash> {
    buckets: Arc<RwLock<HashMap<UtcDate, HashMap<T, u64>>>>,
}
impl<T: Clone + Eq + core::hash::Hash> MonthlyDistributionCounter<T> {
    async fn report_occurrence(&self, date: &UtcDate, occurrence: T) {
        let mut buckets = self.buckets.write().await;
        *buckets
            .entry(date.clone())
            .or_insert(Default::default())
            .entry(occurrence)
            .or_insert(0) += 1;

        let month_ago = Utc::now()
            .date()
            .checked_sub_signed(chrono::Duration::days(30))
            .unwrap();
        buckets.retain(|date, _| date.0 > month_ago);
    }
    async fn list(&self) -> HashMap<UtcDate, HashMap<T, u64>> {
        self.buckets.read().await.clone()
    }
}

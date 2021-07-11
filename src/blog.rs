use crate::utils::*;
use chrono::{Date, Datelike, Utc};
use log::info;
use std::{collections::HashMap, sync::RwLock};

#[derive(Clone, Debug)]
pub struct Article {
    pub key: String,
    pub title: String,
    pub published: Date<Utc>,
    pub content: String,
}

/// A database for articles. It gets articles from the GitHub repo.
pub struct ArticleDb {
    articles: HashMap<String, Article>,
}
impl ArticleDb {
    const BASE_URL: &'static str =
        "https://raw.githubusercontent.com/marcelgarus/server/master/blog";

    pub async fn new() -> Self {
        let mut db = Self {
            articles: Default::default(),
        };
        db.load().await.unwrap();
        db
    }

    async fn load(&mut self) -> Result<(), String> {
        let keys = download(&format!("{}/published.md", Self::BASE_URL))
            .await?
            .split('\n')
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .filter(|line| !line.starts_with('#'))
            .map(article_line::to_key_and_date)
            .filter_map(|it| it.ok())
            .collect::<Vec<(String, Date<Utc>)>>();

        let mut articles: HashMap<String, Article> = Default::default();
        for (key, date) in keys {
            let git_url = format!(
                "{}/{:04}-{:02}-{:02}-{}.md",
                Self::BASE_URL,
                date.year(),
                date.month(),
                date.day(),
                key
            );
            info!("Fetching article from {}", git_url);
            let content = download(&git_url).await?;
            articles.insert(
                key.clone(),
                Article {
                    key,
                    title: "The Title".into(),
                    published: date,
                    content,
                },
            );
        }

        info!(
            "Loaded articles: {}",
            itertools::join(articles.keys(), ", ")
        );
        self.articles = articles;
        Ok(())
    }

    pub fn article_for(&self, key: &str) -> Option<Article> {
        self.articles.get(key).map(|article| article.clone())
    }

    pub fn list(&self) -> Vec<Article> {
        self.articles
            .values()
            .map(|article| article.clone())
            .collect()
    }
}

mod article_line {
    use chrono::{Date, TimeZone, Utc};
    use nom::{
        bytes::complete::tag, character::complete::digit1, combinator::map_res, sequence::tuple,
        IResult,
    };

    pub fn to_key_and_date(line: &str) -> Result<(String, Date<Utc>), String> {
        parse(line)
            .map(|it| it.1)
            .map_err(|err| format!("Error while parsing article id: {:?}", err))
    }
    fn parse(input: &str) -> IResult<&str, (String, Date<Utc>)> {
        let (input, (year, _, month, _, day, _)) = tuple((
            map_res(digit1, |it: &str| it.parse::<i32>()),
            tag("-"),
            map_res(digit1, |it: &str| it.parse::<u32>()),
            tag("-"),
            map_res(digit1, |it: &str| it.parse::<u32>()),
            tag("-"),
        ))(input)?;
        Ok(("", (input.to_owned(), Utc.ymd(year, month, day))))
    }
}

/// Articles look like this: GET /article-key
/// You can get an overview of all articles like this: GET/blog
pub struct Handler {
    db: RwLock<ArticleDb>,
}
impl Handler {
    pub async fn new() -> Self {
        Self {
            db: RwLock::new(ArticleDb::new().await),
        }
    }
    pub async fn handle(&self, request: &Request) -> Option<Response> {
        if request.method == Method::GET {
            if request.path.len() != 1 {
                return None;
            }
            let key: String = request.path.get(0).unwrap().into();
            let article = self.db.read().unwrap().article_for(&key)?;
            info!("Reading article {}", article.key);
            return Some(hyper::Response::with_body(article.content.into()));
        }

        None
    }
}

use crate::utils::*;
use comrak::{
    arena_tree::Children,
    nodes::{Ast, AstNode, ListType, NodeValue},
    parse_document, Arena, ComrakOptions,
};
use itertools::Itertools;
use log::{info, warn};
use rand::prelude::SliceRandom;
use serde::Deserialize;
use std::fs;
use std::{cell::RefCell, collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::RwLock;

type Date = chrono::Date<chrono::Utc>;

#[derive(Clone, Debug)]
pub struct Article {
    pub key: String,
    pub title: String,
    pub published: Option<Date>,
    pub read_duration: Duration,
    pub topics: Vec<String>,
    pub description: String,
    pub teaser: String,
    pub content: String,
    pub next: Option<String>,
}

/// A database for articles. It gets articles from the GitHub repo.
#[derive(Clone, Default)]
pub struct Blog {
    /// All articles by key.
    articles: Arc<RwLock<HashMap<String, Article>>>,

    /// Keys of articles with timestamps in the order that they appeared in the
    /// `published.md` file.
    article_keys: Arc<RwLock<Vec<String>>>,

    /// Topics and article counts for each topic.
    all_topics: Arc<RwLock<Vec<(String, usize)>>>,
}
impl Blog {
    const BASE_PATH: &'static str = "blog";

    pub async fn new() -> Self {
        let db = Blog::default();
        db.load_from_filesystem().await.unwrap();
        db
    }

    pub async fn load_from_filesystem(&self) -> Result<(), String> {
        let articles: HashMap<String, Article> = fs::read_dir(Self::BASE_PATH)
            .expect("Couldn't read blog articles.")
            .filter_map(|it| it.ok())
            .filter_map(|entry| {
                let name = entry.file_name().into_string().ok()?;
                let (key, date) = article_line::parse(&name)?;
                Some((key, date, entry.path()))
            })
            .map(|(key, date, path)| {
                info!("Loading article {}", key);
                let content = String::from_utf8(
                    fs::read(path).expect(&format!("Can't read article {}.", key)),
                )
                .expect(&format!("Article {} contains non-UTF8 chars.", key));
                let article = Article::from_key_and_date_and_markdown(key.clone(), date, &content);
                (key, article)
            })
            .collect();

        let keys = articles
            .values()
            .filter(|it| it.published.is_some())
            .sorted_by_key(|it| it.published)
            .map(|it| it.key.clone())
            .collect_vec();

        let mut topics = HashMap::<String, usize>::new();
        for article in articles.values() {
            if article.published.is_some() {
                for topic in &article.topics {
                    *topics.entry(topic.clone()).or_insert(0) += 1;
                }
            }
        }
        let mut topics = topics.into_iter().collect_vec();
        topics.sort_by_key(|(_, count)| -(*count as i64));
        info!("Found the following topics: {:?}", topics);

        *self.articles.write().await = articles;
        *self.article_keys.write().await = keys;
        *self.all_topics.write().await = topics;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Option<Article> {
        self.articles
            .read()
            .await
            .get(key)
            .map(|article| article.clone())
    }

    pub async fn get_suggestion_for(&self, article: &Article) -> Article {
        self.get(&self.get_suggestion_key_for(article).await)
            .await
            .unwrap()
    }
    async fn get_suggestion_key_for(&self, article: &Article) -> String {
        // If the article has a suggestion, use that.
        if let Some(next) = &article.next {
            return next.to_string();
        }

        // Otherwise, suggest an article with the same topic.
        let topics = &article.topics;
        let articles_about_the_same_topic = self
            .list()
            .await
            .into_iter()
            .filter(|article| topics.iter().any(|topic| article.matches_topic(topic)))
            .filter(|it| it.key != article.key)
            .collect_vec();
        if !articles_about_the_same_topic.is_empty() {
            return articles_about_the_same_topic
                .choose(&mut rand::thread_rng())
                .expect("There's only one article, so we can't find a suggestion.")
                .key
                .to_string();
        }

        // If the newest article doesn't have a pre-defined suggestion, choose
        // a random one.
        let mut keys = self.article_keys.read().await.clone();
        let index = keys.iter().position(|it| it == &article.key);
        if let Some(index) = index {
            keys.remove(index);
        }
        keys.choose(&mut rand::thread_rng())
            .expect("There's only one article, so we can't find a suggestion.")
            .to_string()
    }

    pub async fn list(&self) -> Vec<Article> {
        let mut articles = vec![];
        for key in self.article_keys.read().await.iter() {
            articles.push(self.get(key).await.unwrap())
        }
        articles.into_iter().rev().collect_vec()
    }

    pub async fn topics(&self) -> Vec<(String, usize)> {
        self.all_topics
            .read()
            .await
            .iter()
            .map(|it| it.clone())
            .collect()
    }
}

mod article_line {
    use super::Date;
    use chrono::{TimeZone, Utc};
    use nom::{
        bytes::complete::tag, character::complete::digit1, combinator::map_res, sequence::tuple,
    };

    pub fn parse(filename: &str) -> Option<(String, Option<Date>)> {
        let mut with_date_parser = tuple::<_, _, (), _>((
            map_res(digit1, |it: &str| it.parse::<i32>()),
            tag("-"),
            map_res(digit1, |it: &str| it.parse::<u32>()),
            tag("-"),
            map_res(digit1, |it: &str| it.parse::<u32>()),
            tag("-"),
        ));
        let timeless_parser = tag::<_, _, ()>("timeless-");
        let draft_parser = tag::<_, _, ()>("draft-");

        let filename = filename.trim_end_matches(".md");
        if let Ok((key, (year, _, month, _, day, _))) = with_date_parser(filename) {
            return Some((key.to_owned(), Some(Utc.ymd(year, month, day))));
        }
        if let Ok((key, _)) = timeless_parser(filename) {
            return Some((key.to_owned(), None));
        }
        if let Ok((key, _)) = draft_parser(filename) {
            return Some((key.to_owned(), None));
        }
        None
    }
}

#[derive(Deserialize, Default)]
struct Config {
    topics: Vec<String>,
    next: Option<String>,
    description: Option<String>,
    read_minutes: Option<usize>,
}

impl Article {
    fn from_key_and_date_and_markdown(key: String, date: Option<Date>, markdown: &str) -> Self {
        let (config, markdown) = {
            let mut parts = markdown.split("--start--").collect_vec();
            let markdown = parts.pop().expect("Article has no content.");
            let config = parts
                .pop()
                .map(|it| toml::from_str(it).expect("Start section is not valid."))
                .unwrap_or(Config::default());
            (config, markdown)
        };

        let arena = Arena::new();
        let mut options = ComrakOptions::default();
        options.extension.footnotes = true;
        options.extension.strikethrough = true;
        let root = parse_document(&arena, &markdown.replace("--snip--", ""), &options);

        let read_duration = config
            .read_minutes
            .map(|minutes| Duration::from_secs(60 * minutes as u64))
            .unwrap_or_else(|| {
                // To estimate the read time, I timed how long it took to read the
                // articles and related that to their Markdown file size. That's not
                // an exact metric by any point (for example, links with long URLs
                // would increase the size without impacting the reading duration),
                // but it's definitely a metric that's not totally garbage.
                //
                // article             | read time | file size
                // --------------------+-----------+----------
                // chest-intro         |  1:05 min |    1400 B
                // chest-chunky        |  6:07 min |    6190 B
                // no-dark-mode-toggle |  2:58 min |    3700 B
                // --------------------+-----------+----------
                // sum                 | 10:10 min |   12290 B
                // Average reading speed: 20.1 bytes per second
                let seconds_per_byte = Duration::from_secs(10 * 60 + 10) / 12290;
                seconds_per_byte * (markdown.len() as u32)
            });

        let teaser = parse_document(
            &arena,
            markdown.split("--snip--").next().unwrap(),
            &ComrakOptions::default(),
        )
        .to_html();

        Self {
            key: key.clone(),
            title: root
                .find_title()
                .expect(&format!("Article \"{}\" contains no title", key)),
            published: date,
            read_duration,
            topics: config.topics,
            description: config.description.unwrap_or_else(|| teaser.strip_html()),
            teaser,
            content: root.to_html(),
            next: config.next,
        }
    }
}
trait FindTitle<'a> {
    fn find_title(&'a self) -> Option<String>;
}
impl<'a> FindTitle<'a> for AstNode<'a> {
    fn find_title(&'a self) -> Option<String> {
        if let NodeValue::Heading(heading) = self.data.borrow().value.clone() {
            if heading.level == 1 {
                let mut output = vec![];
                self.children().to_html_parts(&mut output);
                return Some(itertools::join(output, ""));
            }
        }
        self.children().find_title()
    }
}
trait FindTitleInChildren<'a> {
    fn find_title(self) -> Option<String>;
}
impl<'a> FindTitleInChildren<'a> for Children<'a, RefCell<Ast>> {
    fn find_title(self) -> Option<String> {
        for child in self.collect::<Vec<_>>().clone() {
            if let Some(title) = child.find_title() {
                return Some(title);
            }
        }
        None
    }
}
trait ToHtml<'a> {
    fn to_html(&'a self) -> String;
    fn to_html_parts(&'a self, output: &mut Vec<String>);
}
impl<'a> ToHtml<'a> for AstNode<'a> {
    fn to_html(&'a self) -> String {
        let mut html = vec![];
        self.to_html_parts(&mut html);
        itertools::join(html, "")
    }
    fn to_html_parts(&'a self, output: &mut Vec<String>) {
        match self.data.borrow().value.clone() {
            NodeValue::Document => self.children().to_html_parts(output),
            NodeValue::Heading(heading) => {
                if heading.level == 1 {
                    return; // The title of the entire article is treated separately.
                }
                let tag = format!("h{}", heading.level);
                output.start_tag(&tag);
                self.children().to_html_parts(output);
                output.end_tag(&tag);
            }
            NodeValue::Paragraph => {
                output.start_tag("p");
                self.children().to_html_parts(output);
                output.end_tag("p");
            }
            NodeValue::Text(text) => output.push(text.utf8_or_panic().html_encode()),
            NodeValue::SoftBreak => output.push(" ".into()),
            NodeValue::LineBreak => output.push("<br />".into()),
            NodeValue::Emph => {
                output.start_tag("em");
                self.children().to_html_parts(output);
                output.end_tag("em");
            }
            NodeValue::Strong => {
                output.start_tag("strong");
                self.children().to_html_parts(output);
                output.end_tag("strong");
            }
            NodeValue::Strikethrough => {
                output.start_tag("s");
                self.children().to_html_parts(output);
                output.end_tag("s");
            }
            NodeValue::List(list) => {
                let tag = match list.list_type {
                    ListType::Bullet => "ul",
                    ListType::Ordered => "ol",
                }
                .to_owned();
                output.start_tag(&tag);
                self.children().to_html_parts(output);
                output.end_tag(&tag);
            }
            NodeValue::Item(_) => {
                output.start_tag("li");
                self.children().to_html_parts(output);
                output.end_tag("li");
            }
            NodeValue::HtmlBlock(it) => output.push(format!("{}", it.literal.utf8_or_panic())),
            NodeValue::HtmlInline(it) => output.push(format!("{}", it.utf8_or_panic())),
            NodeValue::ThematicBreak => output.push("<hr />".into()),
            NodeValue::Link(link) => {
                output.start_tag(&format!("a href=\"{}\"", link.url.utf8_or_panic(),));
                output.push(link.title.utf8_or_panic().html_encode());
                self.children().to_html_parts(output);
                output.end_tag("a");
            }
            NodeValue::Image(image) => {
                output.start_tag("center");
                let mut title = vec![];
                self.children().to_html_parts(&mut title);
                let title = title.join("");
                let (is_invertable, title) = if title.starts_with("invert:") {
                    (true, title["invert:".len()..].to_string())
                } else {
                    (false, title)
                };
                output.push(format!(
                    "<img src=\"{}\" alt=\"{}\" class=\"{}\" />",
                    image.url.utf8_or_panic(),
                    title,
                    if is_invertable {
                        "invertable-image"
                    } else {
                        ""
                    }
                ));
                output.end_tag("center");
            }
            NodeValue::Code(code) => {
                let code = code.utf8_or_panic();
                let (code, language) = if code.split(":").count() >= 2 {
                    let language = code.split(":").next().unwrap();
                    (code[language.len() + 1..].into(), language)
                } else {
                    (code, "text")
                };
                output.start_tag(&format!("code class=\"language-{}\"", language));
                output.push(code.html_encode());
                output.end_tag("code");
            }
            NodeValue::CodeBlock(code) => {
                output.start_tag("pre");
                output.start_tag(&format!(
                    "code class=\"language-{}\"",
                    match code.info.utf8_or_panic().as_ref() {
                        "" => "text".into(),
                        other_language => other_language,
                    },
                ));
                output.push(code.literal.utf8_or_panic().html_encode());
                output.end_tag("code");
                output.end_tag("pre");
            }
            NodeValue::BlockQuote => {
                output.start_tag("blockquote");
                self.children().to_html_parts(output);
                output.end_tag("blockquote");
            }
            NodeValue::FootnoteReference(key) => {
                output.push(format!(
                    "<a href=\"#footnote-{}\" class=\"footnote-key\">{}</a>",
                    key.clone().utf8_or_panic(),
                    key.utf8_or_panic()
                ));
            }
            NodeValue::FootnoteDefinition(key) => {
                output.push(format!(
                    "<div id=\"footnote-{}\" class=\"footnote-def\">",
                    key.clone().utf8_or_panic()
                ));
                output.push(format!(
                    "<div class=\"footnote-def-key\"><span class=\"footnote-key\">{}</span></div>",
                    key.utf8_or_panic()
                ));
                output.push("<div class=\"footnote-def-value\">".to_string());
                self.children().to_html_parts(output);
                output.end_tag("div");
                output.end_tag("div");
            }
            _ => warn!("Not handling node {:?} yet.", self),
        }
    }
}
trait ChildrenToHtmlParts<'a> {
    fn to_html_parts(self, output: &mut Vec<String>);
}
impl<'a> ChildrenToHtmlParts<'a> for Children<'a, RefCell<Ast>> {
    fn to_html_parts(self, output: &mut Vec<String>) {
        for child in self.collect::<Vec<_>>().clone() {
            child.to_html_parts(output)
        }
    }
}
trait TagVecExt {
    fn start_tag(&mut self, tag: &str);
    fn end_tag(&mut self, tag: &str);
}
impl TagVecExt for Vec<String> {
    fn start_tag(&mut self, tag: &str) {
        self.push(format!("<{}>", tag));
    }
    fn end_tag(&mut self, tag: &str) {
        self.push(format!("</{}>", tag));
    }
}

pub fn canonicalize_topic(topic: &str) -> String {
    topic.to_ascii_lowercase().replace(" ", "-")
}

impl Article {
    pub fn matches_topic(&self, topic: &str) -> bool {
        self.topics.iter().any(|it| it == topic)
    }
}

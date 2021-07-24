use crate::utils::*;
use chrono::{Date, Datelike, Utc};
use comrak::{
    arena_tree::Children,
    nodes::{Ast, AstNode, ListType, NodeValue},
    parse_document, Arena, ComrakOptions,
};
use log::{info, warn};
use std::{cell::RefCell, collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct Article {
    pub key: String,
    pub title: String,
    pub published: Date<Utc>,
    pub content: String,
    pub teaser: String,
}

/// A database for articles. It gets articles from the GitHub repo.
#[derive(Clone)]
pub struct Blog {
    /// All articles by key.
    articles: Arc<RwLock<HashMap<String, Article>>>,

    /// Article keys in the order that they appeared in the `published.md` file.
    article_keys: Arc<RwLock<Vec<String>>>,
}
impl Blog {
    const BASE_URL: &'static str = "https://raw.githubusercontent.com/marcelgarus/server/main/blog";

    pub async fn new() -> Self {
        let db = Self {
            articles: Default::default(),
            article_keys: Default::default(),
        };
        db.load().await.unwrap();
        db
    }

    pub async fn load(&self) -> Result<(), String> {
        let keys_and_dates = download(&format!("{}/published.md", Self::BASE_URL))
            .await?
            .split('\n')
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .filter(|line| !line.starts_with('#'))
            .filter_map(|line| match article_line::to_key_and_date(line) {
                Ok(it) => Some(it),
                Err(err) => {
                    warn!("Invalid article line {}: {:?}", line, err);
                    None
                }
            })
            .collect::<Vec<(String, Date<Utc>)>>();

        let mut articles: HashMap<String, Article> = Default::default();
        let mut keys = vec![];
        for (key, date) in keys_and_dates {
            let git_url = format!(
                "{}/{:04}-{:02}-{:02}-{}.md",
                Self::BASE_URL,
                date.year(),
                date.month(),
                date.day(),
                key,
            );
            let content = download(&git_url).await?;
            articles.insert(
                key.clone(),
                Article::from_key_and_date_and_markdown(key.clone(), date, &content),
            );
            keys.push(key);
        }

        info!("Loaded articles: {}", itertools::join(&keys, ", "));
        *self.articles.write().await = articles;
        *self.article_keys.write().await = keys;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Option<Article> {
        self.articles
            .read()
            .await
            .get(key)
            .map(|article| article.clone())
    }

    pub async fn list(&self) -> Vec<Article> {
        let mut articles = vec![];
        for key in self.article_keys.read().await.iter() {
            articles.push(self.get(key).await.unwrap())
        }
        articles
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

impl Article {
    fn from_key_and_date_and_markdown(key: String, date: Date<Utc>, markdown: &str) -> Self {
        let arena = Arena::new();
        let root = parse_document(
            &arena,
            &markdown.replace("--snip--", ""),
            &ComrakOptions::default(),
        );

        Self {
            key,
            title: root.find_title().expect("Blog contains no title"),
            published: date,
            content: root.to_html(),
            teaser: parse_document(
                &arena,
                markdown.split("--snip--").next().unwrap(),
                &ComrakOptions::default(),
            )
            .to_html(),
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
        use NodeValue::*;
        match self.data.borrow().value.clone() {
            Document => self.children().to_html_parts(output),
            Heading(heading) => {
                if heading.level == 1 {
                    return; // The title of the entire article is treated separately.
                }
                let tag = format!("h{}", heading.level);
                output.start_tag(&tag);
                self.children().to_html_parts(output);
                output.end_tag(&tag);
            }
            Paragraph => {
                output.start_tag("p");
                self.children().to_html_parts(output);
                output.end_tag("p");
            }
            Text(text) => output.push(text.utf8_or_panic().html_encode()),
            SoftBreak => output.push(" ".into()),
            LineBreak => output.push("<br />".into()),
            Emph => {
                output.start_tag("em");
                self.children().to_html_parts(output);
                output.end_tag("em");
            }
            Strong => {
                output.start_tag("strong");
                self.children().to_html_parts(output);
                output.end_tag("strong");
            }
            List(list) => {
                let tag = match list.list_type {
                    ListType::Bullet => "ul",
                    ListType::Ordered => "ol",
                }
                .to_owned();
                output.start_tag(&tag);
                self.children().to_html_parts(output);
                output.end_tag(&tag);
            }
            Item(_) => {
                output.start_tag("li");
                self.children().to_html_parts(output);
                output.end_tag("li");
            }
            HtmlBlock(it) => output.push(format!("{}", it.literal.utf8_or_panic())),
            ThematicBreak => output.push("<hr />".into()),
            Link(link) => {
                output.start_tag(&format!("a href=\"{}\"", link.url.utf8_or_panic(),));
                output.push(link.title.utf8_or_panic().html_encode());
                self.children().to_html_parts(output);
                output.end_tag("a");
            }
            Image(image) => {
                output.start_tag("center");
                output.push(format!(
                    "<img src=\"{}\" alt=\"{}\" />",
                    image.url.utf8_or_panic(),
                    image.title.utf8_or_panic()
                ));
                output.end_tag("center");
            }
            Code(code) => {
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
            CodeBlock(code) => {
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
            _ => {
                warn!("Not handling node {:?} yet.", self);
            }
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

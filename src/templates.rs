use crate::{
    blog::{canonicalize_topic, Article},
    utils::*,
};
use http::StatusCode;
use itertools::Itertools;
use tokio::fs;

async fn page(title: &str, metadata: &str, content: &str) -> String {
    fs::read_to_string("assets/page.html")
        .await
        .unwrap()
        .replace("{{title}}", title)
        .replace("{{metadata}}", metadata)
        .replace("{{content}}", content)
}

fn metadata(
    site_type: &str,
    canonical_url: &str,
    title: &str,
    description: &str,
    image_src_and_alt: Option<(String, String)>,
) -> String {
    let general_metadata = r#"
            <meta name="title" content="{{title}}" />
            <meta name="description" content="{{description}}" />
            <link rel="canonical" href="{{canonical-url}}" />
            <meta property="og:title" content="{{title}}" />
            <meta property="og:description" content="{{description}}" />
            <meta property="og:url" content="{{canonical-url}}" />
            <meta property="og:site" content="Marcel Garus" />
            <meta property="og:site_name" content="Marcel Garus" />
            <meta property="og:locale" content="en_US" />
            <meta property="og:type" content="{{site-type}}" />
            <meta property="twitter:title" content="{{title}}" />
            <meta property="twitter:description" content="{{description}}" />
            <meta property="twitter:site" content="@MarcelGarus" />
            <meta property="twitter:url" content="{{canonical-url}}" />
        "#;
    let image_metadata = r#"
            <link rel="image_src" href="{{image}}" />
            <meta property="og:image" content="{{image}}" />
            <meta property="og:image:alt" content="{{alt}}" />
            <meta property="twitter:image" content="{{image}}" />
            <meta property="twitter:image:alt" content="{{alt}}" />
        "#;
    itertools::join(
        &vec![
            general_metadata
                .replace("{{site-type}}", site_type)
                .replace("{{canonical-url}}", canonical_url)
                .replace("{{title}}", title)
                .replace("{{description}}", description),
            if let Some((src, alt)) = image_src_and_alt {
                image_metadata
                    .replace("{{image}}", &src)
                    .replace("{{alt}}", &alt)
                    .to_owned()
            } else {
                "".to_owned()
            },
        ],
        "",
    )
}

fn topic(topic: &str) -> String {
    format!(
        "<a href=\"/articles/{}\" class=\"topic\">{}</a>",
        canonicalize_topic(topic),
        topic
    )
}

async fn article_teaser(article: &Article) -> String {
    fs::read_to_string("assets/article-teaser.html")
        .await
        .unwrap()
        .fill_in_article(&article)
}

pub async fn blog_page(articles: Vec<Article>) -> String {
    let mut teasers = vec![];
    for article in articles {
        teasers.push(article_teaser(&article).await);
    }
    page(
        "Blog",
        &metadata(
            "website",
            "https://marcelgarus.dev",
            "Blog",
            "Marcel Garus is a student at the Hasso Plattner Institute in Potsdam and an open source developer mainly using Rust and Flutter.",
            Some(("https://marcelgarus.dev/me.png".into(), "A portrait of me.".into())),
        ),
        &teasers.join("\n"),
    )
    .await
}

pub async fn article_page(article: &Article, suggestion: &Article) -> String {
    page(
        &article.title,
        &metadata(
            "article",
            &format!("https://marcelgarus.dev/{}", article.key),
            &article.title,
            &article.teaser.strip_html(),
            None, // TODO: Add first image.
        ),
        &fs::read_to_string("assets/article-full.html")
            .await
            .unwrap()
            .fill_in_article(&article)
            .replace(
                "{{topics}}",
                &article
                    .topics
                    .iter()
                    .map(|it| topic(it))
                    .natural_join()
                    .unwrap_or("interesting topics".to_string()),
            )
            .replace("{{suggestion}}", &article_teaser(&suggestion).await),
    )
    .await
}

async fn timeline(intro: &str, articles: &[Article], outro: &str) -> String {
    let mut timeline_entries = vec![];
    for article in articles {
        timeline_entries.push(
            fs::read_to_string("assets/timeline-article.html")
                .await
                .unwrap()
                .fill_in_article(article)
                .replace(
                    "{{topics}}",
                    &article.topics.iter().map(|it| topic(it)).join(", "),
                ),
        );
    }
    fs::read_to_string("assets/timeline.html")
        .await
        .unwrap()
        .replace("{{intro}}", intro)
        .replace("{{timeline}}", &timeline_entries.join("\n"))
        .replace("{{outro}}", outro)
}

pub async fn timeline_page(topic: Option<&str>, articles: &[Article]) -> String {
    let timeline = if articles.is_empty() {
        let topic = topic.unwrap();
        timeline(
            &format!("I didn't write about {} yet. If you think that's something I should look into, feel free to <a href=\"/about-me\">contact me</a>", topic),
            &[],
            &format!("Otherwise, you might want to look at <a href=\"/articles\">all articles.</a>"),
        ).await
    } else {
        timeline(
            &match topic {
            Some(topic) => format!("These are my articles about {}:", topic),
            None => "These are all of my articles:".to_string(),
        },
            articles,
            "Didn't find what you were looking for? Checkout <a href=\"/articles\">all articles.</a>"
        ).await
    };
    page(
        "Articles",
        &metadata(
            "website",
            "https://marcelgarus.dev/articles",
            "Articles",
            "A list of articles written by Marcel Garus.",
            None,
        ),
        &timeline,
    )
    .await
}

pub async fn error_page(status_code: StatusCode, title: &str, description: &str) -> String {
    page(
        &format!("{} – {}", status_code, &title),
        &metadata(
            "website",
            "", // TODO
            &title,
            &description,
            None,
        ),
        &fs::read_to_string("assets/error.html")
            .await
            .unwrap()
            .replace("{{title}}", title)
            .replace("{{status}}", &format!("{}", status_code.as_u16()))
            .replace("{{description}}", description),
    )
    .await
}

pub async fn rss_feed(articles: &[Article]) -> String {
    let mut articles_xml = vec![];
    for article in articles {
        articles_xml.push(
            fs::read_to_string("assets/rss-article.xml")
                .await
                .unwrap()
                .fill_in_article(&article),
        );
    }
    fs::read_to_string("assets/rss-feed.xml")
        .await
        .unwrap()
        .replace("{{content}}", &itertools::join(articles_xml, "\n"))
}

trait FillInArticle {
    fn fill_in_article(&self, article: &Article) -> Self;
}
impl FillInArticle for String {
    fn fill_in_article(&self, article: &Article) -> Self {
        let read_minutes =
            (((article.read_duration.as_secs() as f64) / 60.0).round() as u64).max(1);
        let published = article
            .published
            .map(|date| format!("{}", date.format("%Y-%m-%d")));
        let mut infos = vec![];
        if let Some(date) = published.clone() {
            infos.push(date);
        }
        infos.push(format!("{} minute read", read_minutes,));
        infos.append(&mut article.topics.iter().map(|it| topic(it)).collect_vec());

        self.replace("{{key}}", &article.key)
            .replace("{{title}}", &article.title)
            .replace("{{published}}", &published.unwrap_or("unknown".into()))
            .replace("{{info}}", &itertools::join(infos.into_iter(), " · "))
            .replace("{{teaser}}", &article.teaser)
            .replace("{{body}}", &article.content)
            .replace("{{read-minutes}}", &format!("{}", read_minutes))
    }
}

trait IterExt {
    fn natural_join(self) -> Option<String>;
}
impl<I: Iterator<Item = String>> IterExt for I {
    fn natural_join(self) -> Option<String> {
        let all = self.collect_vec();
        let length = all.len();
        Some(match length {
            0 => return None,
            1 => all.into_iter().next().unwrap(),
            2 => format!("{} and {}", all[0], all[1]),
            _ => {
                let mut parts = vec![];
                for (i, item) in all.into_iter().enumerate() {
                    let is_last = i == length - 1;
                    if !parts.is_empty() {
                        parts.push(if is_last { ", and " } else { ", " }.to_string());
                    }
                    parts.push(item);
                }
                parts.join("")
            }
        })
    }
}

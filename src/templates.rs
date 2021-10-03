use crate::{blog::Article, utils::*};
use http::StatusCode;
use tokio::fs;

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
            Some(("https://marcelgarus/me.png".into(), "A portrait of me.".into())),
        ),
        &itertools::join(teasers, "\n"),
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
        &article_full(&article, suggestion).await,
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
        &error(status_code, title, description).await,
    )
    .await
}

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
            <meta property="twitter:site" content="@marcelgarus" />
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

async fn article_teaser(article: &Article) -> String {
    fs::read_to_string("assets/article-teaser.html")
        .await
        .unwrap()
        .fill_in_article(&article)
}

async fn article_full(article: &Article, suggestion: &Article) -> String {
    fs::read_to_string("assets/article-full.html")
        .await
        .unwrap()
        .fill_in_article(&article)
        .replace("{{suggestion-key}}", &suggestion.key)
        .replace("{{suggestion-title}}", &suggestion.title)
}

pub async fn rss_feed(articles: &[Article]) -> String {
    let mut articles_xml = vec![];
    for article in articles {
        articles_xml.push(rss_article(&article).await);
    }
    fs::read_to_string("assets/rss-feed.xml")
        .await
        .unwrap()
        .replace("{{content}}", &itertools::join(articles_xml, "\n"))
}

async fn rss_article(article: &Article) -> String {
    fs::read_to_string("assets/rss-article.xml")
        .await
        .unwrap()
        .fill_in_article(&article)
}

async fn error(status_code: StatusCode, title: &str, description: &str) -> String {
    fs::read_to_string("assets/error.html")
        .await
        .unwrap()
        .replace("{{title}}", title)
        .replace("{{status}}", &format!("{}", status_code.as_u16()))
        .replace("{{description}}", description)
}

trait FillInArticle {
    fn fill_in_article(&self, article: &Article) -> Self;
}
impl FillInArticle for String {
    fn fill_in_article(&self, article: &Article) -> Self {
        let published = article
            .published
            .map(|date| format!("{}", date.format("%Y-%m-%d")));
        let mut infos = vec![];
        if let Some(date) = published.clone() {
            infos.push(date);
        }
        infos.push(format!(
            "{} minute read",
            (((article.read_duration.as_secs() as f64) / 60.0).round() as u64).max(1),
        ));

        self.replace("{{key}}", &article.key)
            .replace("{{title}}", &article.title)
            .replace("{{published}}", &published.unwrap_or("unknown".into()))
            .replace("{{info}}", &itertools::join(infos.into_iter(), " · "))
            .replace("{{teaser}}", &article.teaser)
            .replace("{{body}}", &article.content)
    }
}

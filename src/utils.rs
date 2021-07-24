use actix_web::{body::AnyBody, http::HeaderMap, HttpResponse, HttpResponseBuilder};
use http::{HeaderValue, StatusCode};

use crate::blog::Article;

pub trait VecStringExt {
    fn clone_first_n(&self, n: usize) -> Option<Vec<String>>;
    fn starts_with(&self, other: Vec<&str>) -> bool;
    fn clone_except_first(&self, n: usize) -> Vec<String>;
}
impl VecStringExt for Vec<String> {
    fn clone_first_n(&self, n: usize) -> Option<Vec<String>> {
        if self.len() < n {
            None
        } else {
            Some(self.iter().take(n).map(|s| s.clone()).collect())
        }
    }
    fn starts_with(&self, other: Vec<&str>) -> bool {
        self.iter().zip(other).all(|(a, b)| a == b)
    }
    fn clone_except_first(&self, n: usize) -> Vec<String> {
        self.iter().skip(n).map(|s| s.clone()).collect()
    }
}

pub trait Utf8OrPanicExt {
    fn utf8_or_panic(self) -> String;
}
impl Utf8OrPanicExt for Vec<u8> {
    fn utf8_or_panic(self) -> String {
        String::from_utf8(self).unwrap()
    }
}
impl Utf8OrPanicExt for &[u8] {
    fn utf8_or_panic(self) -> String {
        String::from_utf8(self.to_vec()).unwrap()
    }
}

pub trait GetUtf8HeaderExt {
    fn get_utf8(&self, key: &str) -> Option<String>;
}
impl GetUtf8HeaderExt for HeaderMap {
    fn get_utf8(&self, key: &str) -> Option<String> {
        self.get(key)
            .and_then(|value| String::from_utf8(value.as_bytes().to_vec()).ok())
    }
}

pub trait Utf8OrNoneExt {
    fn utf8_or_none(&self) -> Option<String>;
}
impl Utf8OrNoneExt for HeaderValue {
    fn utf8_or_none(&self) -> Option<String> {
        String::from_utf8(self.as_bytes().to_vec()).ok()
    }
}
impl Utf8OrNoneExt for Option<&HeaderValue> {
    fn utf8_or_none(&self) -> Option<String> {
        self.and_then(|value| value.utf8_or_none())
    }
}

pub trait HtmlEncode {
    fn html_encode(&self) -> Self;
}
impl HtmlEncode for String {
    fn html_encode(&self) -> Self {
        self.replace("&", "&amp;").replace("<", "&lt;")
    }
}

/// Fetches the body from a URL. It should return a 200 code and valid UTF-8 content.
pub async fn download(url: &str) -> Result<String, String> {
    let response = reqwest::get(url)
        .await
        .map_err(|err| format!("Couldn't get {}: {:?}", url, err))?;
    if response.status() != StatusCode::OK {
        return Err(format!(
            "Fetching {} returned a non-200 code: {}",
            url,
            response.status()
        ));
    }
    let content = response
        .bytes()
        .await
        .map_err(|err| format!("Body of {} has invalid bytes: {}.", url, err))?;
    let content = String::from_utf8(content.to_vec())
        .map_err(|_| format!("Body of {} is not UTF-8.", url))?;
    Ok(content)
}

pub mod template {
    use tokio::fs;

    pub async fn page() -> String {
        fs::read_to_string("assets/page.html").await.unwrap()
    }
    pub async fn article_teaser() -> String {
        fs::read_to_string("assets/article-teaser.html")
            .await
            .unwrap()
    }
    pub async fn full_article() -> String {
        fs::read_to_string("assets/article-full.html")
            .await
            .unwrap()
    }
    pub async fn error() -> String {
        fs::read_to_string("assets/error.html").await.unwrap()
    }
}

pub trait FillInTemplateExt {
    fn fill_in_content(&self, content: &str) -> Self;
    fn fill_in_article(&self, article: &Article) -> Self;
    fn fill_in_error(&self, status_code: StatusCode, title: &str, description: &str) -> Self;
}
impl FillInTemplateExt for String {
    fn fill_in_content(&self, content: &str) -> Self {
        self.replace("{{content}}", content)
    }
    fn fill_in_article(&self, article: &Article) -> Self {
        let mut infos = vec![];
        if let Some(date) = article.published {
            infos.push(format!("{}", date.format("%Y-%m-%d")));
        }
        infos.push("5 minute read".into());

        self.replace("{{key}}", &article.key)
            .replace("{{title}}", &article.title)
            .replace("{{info}}", &itertools::join(infos.into_iter(), " · "))
            .replace("{{teaser}}", &article.teaser)
            .replace("{{body}}", &article.content)
    }
    fn fill_in_error(&self, status_code: StatusCode, title: &str, description: &str) -> Self {
        self.replace("{{title}}", title)
            .replace("{{status}}", &format!("{}", status_code.as_u16()))
            .replace("{{description}}", description)
    }
}

pub trait HtmlResponse {
    fn html<B: Into<AnyBody>>(&mut self, body: B) -> HttpResponse<AnyBody>;
}
impl HtmlResponse for HttpResponseBuilder {
    fn html<B: Into<AnyBody>>(&mut self, body: B) -> HttpResponse<AnyBody> {
        self.content_type("text/html").body(body)
    }
}

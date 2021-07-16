// General utilities.

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

pub trait Utf8OrPanicVecExt {
    fn utf8_or_panic(self) -> String;
}
impl Utf8OrPanicVecExt for Vec<u8> {
    fn utf8_or_panic(self) -> String {
        String::from_utf8(self).unwrap()
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
            "Getting {} returned a non-200 code: {}",
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

// Stuff for handlers.

use http::StatusCode;
pub use hyper::{Body, Method};
use log::error;
use url::Url;

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub path: Vec<String>, // The path segments
    pub query_string: String,
    pub user_agent: String,
    pub language: String,
    pub is_admin: bool,
}
impl Request {
    pub fn from(request: &hyper::Request<hyper::Body>) -> Option<Self> {
        // The `request.url()` is a relative one, but the url package needs an absolute one, even
        // though we are only interested in the relative parts. So, we prefix it with some URL known
        // to be valid.
        let url = Url::parse("https://example.net").unwrap().join(
            request
                .uri()
                .path_and_query()
                .map(|it| it.as_str())
                .unwrap_or(""),
        );
        let url = match url {
            Ok(url) => url,
            Err(err) => {
                error!("Couldn't parse URL \"{}\": {}", request.uri(), err);
                return None;
            }
        };

        fn get_header_value(request: &hyper::Request<hyper::Body>, field: &str) -> String {
            request
                .headers()
                .get(field)
                .and_then(|value| value.to_str().ok())
                .unwrap_or("")
                .to_owned()
        }

        Some(Self {
            method: request.method().clone(),
            path: url
                .path_segments()
                .unwrap_or("".split(' '))
                .filter(|segment| !segment.is_empty())
                .map(|segment| segment.to_owned())
                .collect(),
            query_string: url.query().unwrap_or("").to_owned(),
            user_agent: get_header_value(request, "user-agent"),
            language: get_header_value(request, "accept-language"),
            is_admin: true, // TODO
        })
    }
}

pub type Response = hyper::Response<Body>;
pub trait FancyResponse {
    fn builder() -> http::response::Builder;
    fn with_body(body: Body) -> Self;
    fn empty() -> Self;
}
impl FancyResponse for Response {
    fn builder() -> http::response::Builder {
        http::Response::builder()
    }
    fn with_body(body: Body) -> Self {
        hyper::Response::builder().body(body).unwrap()
    }
    fn empty() -> Self {
        Self::with_body("".into())
    }
}
pub trait TrustedBuilder {
    fn trusted_body(self, body: Body) -> Response;
}
impl TrustedBuilder for http::response::Builder {
    fn trusted_body(self, body: Body) -> Response {
        self.body(body).unwrap()
    }
}

/// Simply returns the contents of a file as the response.
pub async fn file_content(path: &str) -> Response {
    // TODO: Make this async
    match std::fs::read(path) {
        Ok(content) => Response::with_body(content.into()),
        Err(_) => server_error_page(&format!("The file {} is missing.", path)).await,
    }
}

pub async fn not_authenticated_page() -> Response {
    error_page(
        401,
        "This action requires authentication, which you don't have.".into(),
    )
    .await
}

pub async fn server_error_page(error: &str) -> Response {
    error_page(500, &format!("This is an internal error: {}", error)).await
}

pub async fn error_page(status: u16, description: &str) -> Response {
    Response::builder().status(status).trusted_body(
        format!(
            "This is an ugly error page. This is the error: {}",
            description
        )
        .into(),
    )
}

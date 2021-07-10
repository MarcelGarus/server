///! In the context of this file, handlers are things that can handle requests to the server. There
///! are two kinds of handlers:
///!
///! * Stateless handlers are just a single function.
///! * Stateful handlers are a struct with a `new` and a `handle` function.
///!
///! Also, some handlers can handle all requests and return a `Response`, others can only handle a
///! subset of requests and return an `Option<Response>`.
use crate::shortcuts::*;
use crate::visits::{Visit, VisitsLog};
pub use hyper::Method;
use log::error;
use std::sync::RwLock;
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
        let url = Url::parse("https://example.net")
            .unwrap()
            .join(request.uri().path());
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
            user_agent: get_header_value(request, "User-Agent"),
            language: get_header_value(request, "Accept-Language"),
            is_admin: true, // TODO
        })
    }
}

#[derive(Debug)]
pub struct Response {
    pub status_code: StatusCode,
    pub body: Vec<u8>,
}
pub type StatusCode = u16;
impl Response {
    pub fn ok(body: Vec<u8>) -> Self {
        Response {
            status_code: 200,
            body,
        }
    }
}

/// The public handle function that handles all requests.
///
/// # Visits API
///
/// The visits API looks like this (it's not exactly following the best practices for RESTful
/// APIs, but having all parameters including the key in the querystrings allows for easier
/// deserialization):
///
/// * GET /api/visits: Returns a list of all visits.
pub struct RootHandler {
    visits: RwLock<VisitsLog>,
    shortcuts: ShortcutsHandler,
}
impl RootHandler {
    pub fn new() -> Self {
        Self {
            visits: RwLock::new(VisitsLog::new()),
            shortcuts: ShortcutsHandler::new(),
        }
    }
    pub fn handle(&self, request: &Request) -> Response {
        let visit = Visit::start(&request);

        let response = static_assets(request)
            .or_else(|| crate::visits::handle(&self.visits, request))
            .or_else(|| self.shortcuts.handle(request))
            .unwrap_or_else(|| {
                error_page(404, "Couldn't find the page you're looking for.".into())
            });

        let visit = visit.end(&response);
        self.visits.write().unwrap().register(visit);
        response
    }
}

/// A handler for general static assets, like the main landing page and the icon.
fn static_assets(request: &Request) -> Option<Response> {
    if request.method == Method::GET && request.path.is_empty() {
        return Some(file_content("index.html"));
    }
    if request.method == Method::GET && request.path == vec!["icon.png"] {
        return Some(file_content("icon.png"));
    }
    return None;
}

/// Simply returns the contents of a file as the response.
fn file_content(path: &str) -> Response {
    match std::fs::read(path) {
        Ok(content) => Response::ok(content),
        Err(_) => server_error_page(&format!("The file {} is missing.", path)),
    }
}

pub fn not_authenticated_page() -> Response {
    error_page(
        401,
        "This action requires authentication, which you don't have.".into(),
    )
}

pub fn server_error_page(error: &str) -> Response {
    error_page(500, &format!("This is an internal error: {}", error))
}

pub fn error_page(status_code: StatusCode, description: &str) -> Response {
    Response {
        status_code,
        body: format!(
            "This is an ugly error page. This is the error: {}",
            description
        )
        .into_bytes(),
    }
}

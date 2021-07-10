use crate::shortcuts::*;
use crate::utils::*;
use crate::visits::Visit;
use crate::visits::VisitsLog;
use hyper::Method;
use log::error;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug)]
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
    fn ok(body: Vec<u8>) -> Self {
        Response {
            status_code: 200,
            body,
        }
    }
}

// In the context of this file, handlers are things that can handle requests to the server. There
// are two kinds of handlers:
//
// * Stateless handlers are just a single function.
// * Stateful handlers are a struct with a `new` and a `handle` function.
//
// Also, some handlers can handle all requests and return a `Response`, others can only handle a
// subset of requests and return an `Option<Response>`.

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
    visits: VisitsLog,
    shortcuts: ShortcutsHandler,
}
impl RootHandler {
    pub fn new() -> Self {
        Self {
            visits: VisitsLog::new(),
            shortcuts: ShortcutsHandler::new(),
        }
    }
    pub fn handle(&mut self, request: &Request) -> Response {
        let visit = Visit::start(&request);

        let response = static_assets(request)
            .or_else(|| self.handle_visits_admin(request))
            .or_else(|| self.shortcuts.handle(request))
            .unwrap_or_else(|| {
                error_page(404, "Couldn't find the page you're looking for.".into())
            });

        let visit = visit.end(&response);
        self.visits.register(visit);
        response
    }
    pub fn handle_visits_admin(&mut self, request: &Request) -> Option<Response> {
        if request.path.starts_with(vec!["api", "visits"]) {
            let rest_of_path: Vec<String> = request.path.clone_except_first(2);
            if !request.is_admin {
                return Some(not_authenticated_page());
            }
            if request.method == Method::GET && rest_of_path.is_empty() {
                return Some(match serde_json::to_string(&self.visits.list()) {
                    Ok(json) => Response::ok(json.into_bytes()),
                    Err(err) => {
                        server_error_page(&format!("Couldn't serialize visits to JSON: {}", err))
                    }
                });
            }
        }

        None
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

/// A handler for shortcuts.
///
/// Shortcuts look like this: GET /go/some-shortcut-key
///
/// The shortcuts API looks like this (it's not exactly following the best practices for RESTful
/// APIs, but having all parameters – including the key – in the query string allows for easier
/// deserialization):
///
/// * GET /api/shortcuts: Returns a list of shortcuts.
/// * POST /api/shortcuts/set?key=foo&url=some-url: Sets a shortcut.
/// * POST /api/shortcuts/delete?key=foo: Deletes a shortcut.
struct ShortcutsHandler {
    db: ShortcutsDb,
}
impl ShortcutsHandler {
    fn new() -> Self {
        Self {
            db: ShortcutsDb::new(),
        }
    }
    fn handle(&mut self, request: &Request) -> Option<Response> {
        if request.method == Method::GET && request.path.starts_with(vec!["go"]) {
            if request.path.len() != 2 {
                return None;
            }
            let key: String = request.path.get(1).unwrap().into();
            let shortcut = self.db.shortcut_for(&key)?;
            return Some(Response {
                status_code: 200,
                body: format!("We should redirect to {}", shortcut.url).into_bytes(),
            });
        }

        if request.path.starts_with(vec!["api", "shortcuts"]) {
            let rest_of_path: Vec<String> = request.path.clone_except_first(2);
            if !request.is_admin {
                return Some(not_authenticated_page());
            }
            if request.method == Method::GET && rest_of_path.is_empty() {
                return Some(match serde_json::to_string(&self.db.list()) {
                    Ok(json) => Response::ok(json.into_bytes()),
                    Err(err) => {
                        server_error_page(&format!("Couldn't serialize shortcuts to JSON: {}", err))
                    }
                });
            }
            if request.method == Method::POST && rest_of_path == vec!["set"] {
                return Some(match serde_qs::from_str(&request.query_string) {
                    Ok(shortcut) => {
                        self.db.register(shortcut);
                        Response::ok(vec![])
                    }
                    Err(err) => error_page(400, &format!("Invalid data: {}", err)),
                });
            }
            if request.method == Method::POST && rest_of_path == vec!["delete"] {
                return Some(match serde_qs::from_str(&request.query_string) {
                    Ok(delete_request) => {
                        let delete_request: ShortcutDeleteRequest = delete_request;
                        self.db.delete(&delete_request.key);
                        Response::ok(vec![])
                    }
                    Err(err) => error_page(400, &format!("Invalid data: {}", err)),
                });
            }
        }

        None
    }
}
#[derive(Serialize, Deserialize)]
struct ShortcutDeleteRequest {
    key: String,
}

/// Simply returns the contents of a file as the response.
fn file_content(path: &str) -> Response {
    match std::fs::read(path) {
        Ok(content) => Response::ok(content),
        Err(_) => server_error_page(&format!("The file {} is missing.", path)),
    }
}

fn not_authenticated_page() -> Response {
    error_page(
        401,
        "This action requires authentication, which you don't have.".into(),
    )
}

fn server_error_page(error: &str) -> Response {
    error_page(500, &format!("This is an internal error: {}", error))
}

fn error_page(status_code: StatusCode, description: &str) -> Response {
    Response {
        status_code,
        body: format!(
            "This is an ugly error page. This is the error: {}",
            description
        )
        .into_bytes(),
    }
}

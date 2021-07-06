use crate::shortcuts::*;
use crate::utils::*;
use log::error;
use serde::{Deserialize, Serialize};
use tiny_http::Method;
use url::Url;

pub struct Request {
    method: Method,
    path: Vec<String>, // The path segments
    querystring: String,
    is_admin: bool,
}
impl Request {
    pub fn from_tiny(request: &tiny_http::Request) -> Option<Self> {
        // The `request.url()` is a relative one, but the url package needs an absolute one, even
        // though we are only interested in the relative parts. So, we prefix it with some URL known
        // to be valid.
        let url = Url::parse("https://example.net")
            .unwrap()
            .join(request.url());
        let url = match url {
            Ok(url) => url,
            Err(err) => {
                error!("Couldn't parse URL \"{}\": {}", request.url(), err);
                return None;
            }
        };
        Some(Self {
            method: request.method().clone(),
            path: url
                .path_segments()
                .map(|segments| segments.map(|segment| segment.to_owned()).collect())
                .unwrap_or(vec![]),
            querystring: url.query().unwrap_or("").to_owned(),
            is_admin: true, // TODO
        })
    }
}
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
pub struct RootHandler {
    shortcuts: ShortcutsHandler,
}
impl RootHandler {
    pub fn new() -> Self {
        Self {
            shortcuts: ShortcutsHandler::new(),
        }
    }
    pub fn handle(&mut self, request: &Request) -> Response {
        static_assets(request)
            .or_else(|| self.shortcuts.handle(request))
            .unwrap_or_else(|| error_page(404, "Couldn't find the page you're looking for.".into()))
    }
}

/// A handler for general static assets, like the main landing page and the icon.
fn static_assets(request: &Request) -> Option<Response> {
    if request.method == Method::Get && request.path.is_empty() {
        return Some(file_content("index.html"));
    }
    if request.method == Method::Get && request.path == vec!["icon.png"] {
        return Some(file_content("icon.png"));
    }
    return None;
}

/// A handler for shortcuts.
///
/// Shortcuts look like this: GET /go/some-shortcut-key
///
/// The shortcuts API looks like this (it's not exactly following the best practices for RESTful
/// APIs, but having all parameters including the key in the querystrings allows for easier
/// deserialization):
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
        if request.method == Method::Get && request.path.starts_with(vec!["go"]) {
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
            if request.method == Method::Get && rest_of_path.is_empty() {
                return Some(match serde_json::to_string(&self.db.list()) {
                    Ok(json) => Response::ok(json.into_bytes()),
                    Err(err) => {
                        server_error_page(&format!("Couldn't serialize shortcuts to JSON: {}", err))
                    }
                });
            }
            if request.method == Method::Post && rest_of_path == vec!["set"] {
                return Some(match serde_qs::from_str(&request.querystring) {
                    Ok(shortcut) => {
                        self.db.register(shortcut);
                        Response::ok(vec![])
                    }
                    Err(err) => error_page(400, &format!("Invalid data: {}", err)),
                });
            }
            if request.method == Method::Post && rest_of_path == vec!["delete"] {
                return Some(match serde_qs::from_str(&request.querystring) {
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

use tiny_http::{Method, Request};

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

/// The public handle function that handles all requests.
pub fn handle(request: &Request) -> Response {
    main_page(request)
        .unwrap_or_else(|| error_page(404, "Couldn't find the page you're looking for.".into()))
}

/// The main landing page with some information about me.
fn main_page(request: &Request) -> Option<Response> {
    Some(match (request.method(), request.url()) {
        (Method::Get, "/") => file_content("index.html"),
        (Method::Get, "/icon.png") => file_content("icon.png"),
        _ => return None,
    })
}

/// Simply returns the contents of a file as the response.
fn file_content(path: &str) -> Response {
    match std::fs::read(path) {
        Ok(content) => Response::ok(content),
        Err(_) => error_page(
            500,
            format!(
                "This is an internal server error. The file {} is missing.",
                path
            ),
        ),
    }
}

/// A page with an error message.
fn error_page(status_code: StatusCode, description: String) -> Response {
    Response {
        status_code,
        body: format!(
            "This is an ugly error page. This is the error: {}",
            description
        )
        .into_bytes(),
    }
}

use crate::utils::*;

/// A handler for general static assets, like the main landing page and the icon.
pub fn handle(request: &Request) -> Option<Response> {
    if request.method == Method::GET && request.path.is_empty() {
        return Some(file_content("index.html"));
    }
    if request.method == Method::GET && request.path == vec!["icon.png"] {
        return Some(file_content("icon.png"));
    }
    return None;
}

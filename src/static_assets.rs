use crate::utils::*;

/// A handler for general static assets, like the main landing page and the icon.
pub async fn handle(request: &Request) -> Option<Response> {
    if request.method == Method::GET && request.path.is_empty() {
        return Some(file_content("index.html").await);
    }
    if request.method == Method::GET && request.path == vec!["icon.png"] {
        return Some(file_content("icon.png").await);
    }
    return None;
}

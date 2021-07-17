use crate::utils::*;

/// A handler for general static assets, like the main landing page and the icon.
pub async fn handle(request: &Request) -> Option<Response> {
    if request.method == Method::GET && request.path.is_empty() {
        return Some(file_content("assets/index.html").await);
    }
    if request.method == Method::GET && request.path == vec!["favicon.ico"] {
        return Some(file_content("assets/icon.ico").await);
    }

    let static_assets = vec!["icon.png", "prism.css", "prism.js"];
    for asset in static_assets {
        if request.method == Method::GET && request.path == vec![asset] {
            return Some(file_content(&format!("assets/{}", asset)).await);
        }
    }

    None
}

use crate::utils::*;

/// A handler for general static assets, like the main landing page and the icon.
pub async fn handle(request: &Request) -> Option<Response> {
    if request.method == Method::GET && request.path.is_empty() {
        return Some(file_content("assets/index.html").await);
    }

    let static_assets = vec![
        vec!["article.html"],
        vec!["footer.html"],
        vec!["icon.png"],
        vec!["scripts.js"],
        vec!["style.css"],
    ];
    for asset in static_assets {
        if request.method == Method::GET && request.path == asset {
            return Some(file_content(&format!("assets/{}", itertools::join(asset, "/"))).await);
        }
    }

    None
}

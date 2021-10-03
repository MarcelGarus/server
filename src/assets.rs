use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Asset {
    pub key: String,
    pub path: String,
    pub content_type: String,
}
impl Asset {
    fn internal(file_name: &str, content_type: &str) -> Self {
        Self {
            key: file_name.into(),
            path: format!("assets/{}", file_name),
            content_type: content_type.into(),
        }
    }
    fn file(file_name: &str, content_type: &str) -> Self {
        Self {
            key: file_name.chars().take_while(|c| *c != '.').collect(),
            path: format!("files/{}", file_name),
            content_type: content_type.into(),
        }
    }
}

lazy_static! {
    static ref ASSETS: HashMap<String, Asset> = {
        vec![
            Asset::internal("favicon.ico", "image/vnd.microsoft.icon"),
            Asset::internal("icon.png", "image/png"),
            Asset::internal("me.png", "image/png"),
            Asset::internal("me.webp", "image/webp"),
            Asset::internal("prism.css", "text/css"),
            Asset::internal("prism.js", "text/javascript"),
            Asset::internal("fonts.css", "text/css"),
            Asset::internal("JetBrainsMono-Latin.woff2", "font/woff2"),
            Asset::internal("JetBrainsMono-LatinExt.woff2", "font/woff2"),
            Asset::internal("JosefinSans-Latin.woff2", "font/woff2"),
            Asset::internal("JosefinSans-LatinExt.woff2", "font/woff2"),
            Asset::file("fourierreihe.pdf", "application/pdf"),
        ]
        .into_iter()
        .map(|asset| (asset.key.clone(), asset))
        .collect()
    };
}

pub fn asset_for(key: &str) -> Option<Asset> {
    ASSETS.get(key).map(|shortcut| shortcut.clone())
}

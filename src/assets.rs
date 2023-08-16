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
            Asset::internal("style.css", "text/css"),
            Asset::internal("prism.css", "text/css"),
            Asset::internal("prism.js", "text/javascript"),
            Asset::internal("fonts.css", "text/css"),
            Asset::internal("font-sans-bold-italic-greek.woff2", "font/woff2"),
            Asset::internal("font-sans-bold-italic-latin-ext.woff2", "font/woff2"),
            Asset::internal("font-sans-bold-italic-latin.woff2", "font/woff2"),
            Asset::internal("font-sans-bold-normal-greek.woff2", "font/woff2"),
            Asset::internal("font-sans-bold-normal-latin-ext.woff2", "font/woff2"),
            Asset::internal("font-sans-bold-normal-latin.woff2", "font/woff2"),
            Asset::internal("font-mono-regular-normal-latin-ext.woff2", "font/woff2"),
            Asset::internal("font-mono-regular-normal-latin.woff2", "font/woff2"),
            Asset::internal("font-sans-regular-italic-greek.woff2", "font/woff2"),
            Asset::internal("font-sans-regular-italic-latin-ext.woff2", "font/woff2"),
            Asset::internal("font-sans-regular-italic-latin.woff2", "font/woff2"),
            Asset::internal("font-sans-regular-normal-greek.woff2", "font/woff2"),
            Asset::internal("font-sans-regular-normal-latin-ext.woff2", "font/woff2"),
            Asset::internal("font-sans-regular-normal-latin.woff2", "font/woff2"),
            Asset::internal("font-serif-bold-italic-latin-ext.woff2", "font/woff2"),
            Asset::internal("font-serif-bold-italic-latin.woff2", "font/woff2"),
            Asset::internal("font-serif-bold-normal-latin-ext.woff2", "font/woff2"),
            Asset::internal("font-serif-bold-normal-latin.woff2", "font/woff2"),
            Asset::internal("font-serif-regular-italic-latin-ext.woff2", "font/woff2"),
            Asset::internal("font-serif-regular-italic-latin.woff2", "font/woff2"),
            Asset::internal("font-serif-regular-normal-latin-ext.woff2", "font/woff2"),
            Asset::internal("font-serif-regular-normal-latin.woff2", "font/woff2"),
            Asset::file("coronoise-slides.pdf", "application/pdf"),
            Asset::file("energy-slides.pdf", "application/pdf"),
            Asset::file("federated-learning-slides.pdf", "application/pdf"),
            Asset::file("fourierreihe.pdf", "application/pdf"),
            Asset::file("fuzzing-slides.pdf", "application/pdf"),
            Asset::file("jogging-in-the-cold.mp3", "audio/mpeg"),
            Asset::file("jupyter-energy-slides.pdf", "application/pdf"),
            Asset::file("paralyzer.m4a", "audio/mp4"),
        ]
        .into_iter()
        .map(|asset| (asset.key.clone(), asset))
        .collect()
    };
}

pub fn asset_for(key: &str) -> Option<Asset> {
    ASSETS.get(key).map(|shortcut| shortcut.clone())
}

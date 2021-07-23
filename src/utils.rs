use actix_web::http::HeaderMap;
use http::{HeaderValue, StatusCode};

pub trait VecStringExt {
    fn clone_first_n(&self, n: usize) -> Option<Vec<String>>;
    fn starts_with(&self, other: Vec<&str>) -> bool;
    fn clone_except_first(&self, n: usize) -> Vec<String>;
}
impl VecStringExt for Vec<String> {
    fn clone_first_n(&self, n: usize) -> Option<Vec<String>> {
        if self.len() < n {
            None
        } else {
            Some(self.iter().take(n).map(|s| s.clone()).collect())
        }
    }
    fn starts_with(&self, other: Vec<&str>) -> bool {
        self.iter().zip(other).all(|(a, b)| a == b)
    }
    fn clone_except_first(&self, n: usize) -> Vec<String> {
        self.iter().skip(n).map(|s| s.clone()).collect()
    }
}

pub trait Utf8OrPanicExt {
    fn utf8_or_panic(self) -> String;
}
impl Utf8OrPanicExt for Vec<u8> {
    fn utf8_or_panic(self) -> String {
        String::from_utf8(self).unwrap()
    }
}
impl Utf8OrPanicExt for &[u8] {
    fn utf8_or_panic(self) -> String {
        String::from_utf8(self.to_vec()).unwrap()
    }
}

pub trait GetUtf8HeaderExt {
    fn get_utf8(&self, key: &str) -> Option<String>;
}
impl GetUtf8HeaderExt for HeaderMap {
    fn get_utf8(&self, key: &str) -> Option<String> {
        self.get(key)
            .and_then(|value| String::from_utf8(value.as_bytes().to_vec()).ok())
    }
}

pub trait Utf8OrNoneExt {
    fn utf8_or_none(&self) -> Option<String>;
}
impl Utf8OrNoneExt for HeaderValue {
    fn utf8_or_none(&self) -> Option<String> {
        String::from_utf8(self.as_bytes().to_vec()).ok()
    }
}
impl Utf8OrNoneExt for Option<&HeaderValue> {
    fn utf8_or_none(&self) -> Option<String> {
        self.and_then(|value| value.utf8_or_none())
    }
}

pub trait HtmlEncode {
    fn html_encode(&self) -> Self;
}
impl HtmlEncode for String {
    fn html_encode(&self) -> Self {
        self.replace("&", "&amp;").replace("<", "&lt;")
    }
}

/// Fetches the body from a URL. It should return a 200 code and valid UTF-8 content.
pub async fn download(url: &str) -> Result<String, String> {
    let response = reqwest::get(url)
        .await
        .map_err(|err| format!("Couldn't get {}: {:?}", url, err))?;
    if response.status() != StatusCode::OK {
        return Err(format!(
            "Getting {} returned a non-200 code: {}",
            url,
            response.status()
        ));
    }
    let content = response
        .bytes()
        .await
        .map_err(|err| format!("Body of {} has invalid bytes: {}.", url, err))?;
    let content = String::from_utf8(content.to_vec())
        .map_err(|_| format!("Body of {} is not UTF-8.", url))?;
    Ok(content)
}

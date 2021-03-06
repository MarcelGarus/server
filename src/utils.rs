use actix_web::{body::AnyBody, http::HeaderMap, HttpResponse, HttpResponseBuilder};
use chrono::{Date, Utc};
use http::HeaderValue;
use serde::Serialize;

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

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct UtcDate(pub Date<Utc>);
impl Serialize for UtcDate {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.0.format("%G-%m-%d"))
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

pub trait StripHtml {
    fn strip_html(&self) -> Self;
}
impl StripHtml for String {
    fn strip_html(&self) -> Self {
        let mut b = string_builder::Builder::default();
        let mut skipping = false;
        for c in self.chars() {
            match (skipping, c) {
                (false, '<') => skipping = true,
                (false, c) => b.append(c),
                (true, '>') => skipping = false,
                (true, _) => {}
            }
        }
        b.string().unwrap()
    }
}

pub trait RedirectHttpResponseExt {
    fn redirect_to(location: &str) -> Self;
}
impl RedirectHttpResponseExt for HttpResponse {
    fn redirect_to(location: &str) -> Self {
        HttpResponse::MovedPermanently()
            .append_header(("Location", location))
            .body("")
    }
}

pub trait HtmlResponse {
    fn html<B: Into<AnyBody>>(&mut self, body: B) -> HttpResponse<AnyBody>;
}
impl HtmlResponse for HttpResponseBuilder {
    fn html<B: Into<AnyBody>>(&mut self, body: B) -> HttpResponse<AnyBody> {
        self.content_type("text/html").body(body)
    }
}

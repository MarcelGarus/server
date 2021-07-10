use crate::handlers::*;
use crate::utils::*;
use log::info;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::RwLock};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Shortcut {
    pub key: String,
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct ShortcutsDb {
    shortcuts: HashMap<String, Shortcut>,
}
impl ShortcutsDb {
    pub fn new() -> Self {
        Self {
            shortcuts: Default::default(),
        }
    }

    pub fn shortcut_for(&self, key: &str) -> Option<Shortcut> {
        self.shortcuts.get(key).map(|shortcut| shortcut.clone())
    }

    pub fn list(&self) -> Vec<Shortcut> {
        self.shortcuts
            .values()
            .map(|shortcut| shortcut.clone())
            .collect()
    }

    pub fn register(&mut self, shortcut: Shortcut) {
        info!(
            "The shortcuts in JSON are {}.",
            serde_json::to_string(&self.shortcuts).unwrap()
        );
        self.shortcuts.insert(shortcut.key.clone(), shortcut);
    }

    pub fn delete(&mut self, key: &str) {
        self.shortcuts.remove(key);
    }
}

/// Shortcuts look like this: GET /go/some-shortcut-key
///
/// The shortcuts API looks like this (it's not exactly following the best practices for RESTful
/// APIs, but having all parameters – including the key – in the query string allows for easier
/// deserialization):
///
/// * GET /api/shortcuts: Returns a list of shortcuts.
/// * POST /api/shortcuts/set?key=foo&url=some-url: Sets a shortcut.
/// * POST /api/shortcuts/delete?key=foo: Deletes a shortcut.
pub struct ShortcutsHandler {
    db: RwLock<ShortcutsDb>,
}
impl ShortcutsHandler {
    pub fn new() -> Self {
        Self {
            db: RwLock::new(ShortcutsDb::new()),
        }
    }
    pub fn handle(&self, request: &Request) -> Option<Response> {
        if request.method == Method::GET && request.path.starts_with(vec!["go"]) {
            if request.path.len() != 2 {
                return None;
            }
            let key: String = request.path.get(1).unwrap().into();
            let shortcut = self.db.read().unwrap().shortcut_for(&key)?;
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
            if request.method == Method::GET && rest_of_path.is_empty() {
                return Some(
                    match serde_json::to_string(&self.db.read().unwrap().list()) {
                        Ok(json) => Response::ok(json.into_bytes()),
                        Err(err) => server_error_page(&format!(
                            "Couldn't serialize shortcuts to JSON: {}",
                            err
                        )),
                    },
                );
            }
            if request.method == Method::POST && rest_of_path == vec!["set"] {
                return Some(match serde_qs::from_str(&request.query_string) {
                    Ok(shortcut) => {
                        self.db.write().unwrap().register(shortcut);
                        Response::ok(vec![])
                    }
                    Err(err) => error_page(400, &format!("Invalid data: {}", err)),
                });
            }
            if request.method == Method::POST && rest_of_path == vec!["delete"] {
                return Some(match serde_qs::from_str(&request.query_string) {
                    Ok(delete_request) => {
                        let delete_request: ShortcutDeleteRequest = delete_request;
                        self.db.write().unwrap().delete(&delete_request.key);
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

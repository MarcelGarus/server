use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

use log::info;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::{fs, sync::RwLock};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Shortcut {
    pub key: String,
    pub url: String,
}

/// A database for shortcuts. It simply saves shortcuts into a file.
#[derive(Clone)]
pub struct ShortcutDb {
    shortcuts: Arc<RwLock<HashMap<String, Shortcut>>>,
}
impl ShortcutDb {
    pub async fn new() -> Self {
        let mut shortcuts: HashMap<String, Shortcut> = Default::default();
        if let Ok(json) = fs::read_to_string("shortcuts.json").await {
            let shortcuts_vec: Vec<Shortcut> = serde_json::from_str(&json).unwrap();
            for shortcut in shortcuts_vec {
                shortcuts.insert(shortcut.key.clone(), shortcut);
            }
        }
        info!(
            "Loaded shortcuts: {}",
            itertools::join(shortcuts.keys(), ", ")
        );
        Self {
            shortcuts: Arc::new(RwLock::new(shortcuts)),
        }
    }

    async fn save(&self) {
        fs::write(
            "shortcuts.json",
            serde_json::to_string(&self.shortcuts.read().await.values().collect::<Vec<_>>())
                .unwrap(),
        )
        .await
        .unwrap()
    }

    pub async fn shortcut_for(&self, key: &str) -> Option<Shortcut> {
        self.shortcuts
            .read()
            .await
            .get(key)
            .map(|shortcut| shortcut.clone())
    }

    pub async fn list(&self) -> Vec<Shortcut> {
        self.shortcuts
            .read()
            .await
            .values()
            .map(|shortcut| shortcut.clone())
            .collect()
    }

    pub async fn register(&self, shortcut: Shortcut) {
        self.shortcuts
            .write()
            .await
            .insert(shortcut.key.clone(), shortcut);
        info!(
            "The shortcuts are now {}.",
            serde_json::to_string(&self.shortcuts.read().await.clone()).unwrap()
        );
        self.save().await;
    }

    pub async fn delete(&self, key: &str) {
        self.shortcuts.write().await.remove(key);
        self.save().await;
    }
}

use crate::config::{cg::CG_CONFIG, ENGINE_CONFIG};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use tokio::sync::watch;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct UserData {
    #[serde(default)]
    pub(crate) cg: Vec<u64>,
    #[serde(default)]
    pub(crate) read: HashMap<String, usize>,
}

impl Default for UserData {
    fn default() -> Self {
        let num = CG_CONFIG.length() / 64 + 1;
        Self {
            cg: vec![0; num],
            read: HashMap::new(),
        }
    }
}

impl UserData {
    pub(crate) fn load() -> Self {
        let path = format!("{}/data.toml", ENGINE_CONFIG.save_path());
        let mut data = fs::read_to_string(&path)
            .ok()
            .and_then(|content| toml::from_str::<Self>(&content).ok())
            .unwrap_or_default();
        // 防御：cg 长度不足时补齐，避免解锁时越界
        let num = CG_CONFIG.length() / 64 + 1;
        if data.cg.len() < num {
            data.cg.resize(num, 0);
        }
        data
    }
}

pub(crate) fn spawn_save_task() -> watch::Sender<Option<UserData>> {
    let (tx, mut rx) = watch::channel::<Option<UserData>>(None);
    tokio::spawn(async move {
        while rx.changed().await.is_ok() {
            let snapshot = rx.borrow_and_update().clone();
            if let Some(data) = snapshot {
                let path = format!("{}/data.toml", ENGINE_CONFIG.save_path());
                match toml::to_string(&data) {
                    Ok(content) => {
                        if let Err(e) = tokio::fs::write(&path, content).await {
                            eprintln!("save user data failed: {e}");
                        }
                    }
                    Err(e) => eprintln!("serialize user data failed: {e}"),
                }
            }
        }
    });
    tx
}

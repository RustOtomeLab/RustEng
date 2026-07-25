use crate::config::ENGINE_CONFIG;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};
use tokio::time::Duration;

lazy_static::lazy_static! {
    pub(crate) static ref VOICE_LENGTH: VoiceLength = load_voice();
}

#[derive(Debug, Deserialize)]
struct LengthWrapper {
    cast: HashMap<String, u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct VoiceLength {
    voice_length: HashMap<String, HashMap<String, Duration>>,
}

impl VoiceLength {
    pub(crate) fn find(&self, name: &str) -> Option<&HashMap<String, Duration>> {
        self.voice_length.get(name)
    }
}

fn load_voice() -> VoiceLength {
    let mut voice_length = HashMap::new();
    for char in &ENGINE_CONFIG.character_name_list() {
        let content = fs::read_to_string(format!(
            "{}{}/length.toml",
            ENGINE_CONFIG.voice_path(),
            char
        ))
        .unwrap();
        let item: LengthWrapper = toml::from_str(&content).unwrap();
        voice_length.insert(
            char.to_string(),
            item.cast
                .into_iter()
                .map(|(name, secs)| (name, Duration::from_secs(secs)))
                .collect(),
        );
    }

    VoiceLength { voice_length }
}

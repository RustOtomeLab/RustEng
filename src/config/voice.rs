use crate::config::ENGINE_CONFIG;
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::HashMap, fs};
use tokio::time::Duration;

lazy_static::lazy_static! {
    pub static ref VOICE_LENGTH: VoiceLength = load_voice();
}

#[derive(Debug, Deserialize, Serialize)]
struct Length {
    name: String,
    #[serde(deserialize_with = "deserialize_duration_from_secs")]
    length: Duration,
}

#[derive(Debug, Deserialize, Serialize)]
struct LengthWrapper {
    cast: Vec<Length>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VoiceLength {
    voice_length: HashMap<String, HashMap<String, Duration>>,
}

impl VoiceLength {
    pub fn find(&self, name: &str) -> Option<&HashMap<String, Duration>> {
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
                .map(|length| (length.name, length.length))
                .collect(),
        );
    }

    VoiceLength { voice_length }
}

pub fn deserialize_duration_from_secs<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let seconds = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(seconds))
}

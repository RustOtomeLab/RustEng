use crate::config::voice::deserialize_duration_from_secs;
use crate::config::ENGINE_CONFIG;
use serde::{Deserialize, Serialize};
use std::fs;
use tokio::time::Duration;

lazy_static::lazy_static! {
    pub static ref SCRIPT_CONFIG: ScriptConfig = load_script_config();
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScriptConfig {
    auto: AutoConfig,
}

impl ScriptConfig {
    pub fn delay(&self) -> Duration {
        self.auto.delay
    }

    pub fn is_wait(&self) -> bool {
        self.auto.is_wait
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct AutoConfig {
    #[serde(deserialize_with = "deserialize_duration_from_secs")]
    delay: Duration,
    is_wait: bool,
}

fn load_script_config() -> ScriptConfig {
    let content =
        fs::read_to_string(format!("{}/auto.toml", ENGINE_CONFIG.script_path(),)).unwrap();
    toml::from_str(&content).unwrap()
}

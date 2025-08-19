use serde::{Deserialize, Serialize};
use std::fs;
use crate::error::EngineError;

lazy_static::lazy_static! {
    pub static ref ENGINE_CONFIG: EngineConfig = load_engine_config();
}

#[derive(Debug, Deserialize, Serialize)]
struct InitializeConfig {
    script_path: String,
    background_path: String,
    voice_path: String,
    bgm_path: String,
    figure_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EngineConfig {
    initialize: InitializeConfig,
}

impl EngineConfig {
    pub fn script_path(&self) -> &str {
        &self.initialize.script_path
    }

    pub fn background_path(&self) -> &str {
        &self.initialize.background_path
    }

    pub fn voice_path(&self) -> &str {
        &self.initialize.voice_path
    }

    pub fn bgm_path(&self) -> &str {
        &self.initialize.bgm_path
    }

    pub fn figure_path(&self) -> &str {
        &self.initialize.figure_path
    }
}

fn load_engine_config() -> EngineConfig {
    let content = fs::read_to_string("./source/ini.toml").unwrap();
    toml::from_str(&content).unwrap()
}
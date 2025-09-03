use crate::config::initialize::{Character, InitializeConfig};
use crate::config::volume::VolumeConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

pub mod figure;
pub mod initialize;
pub mod save_load;

pub mod script;
pub mod voice;
pub mod volume;

lazy_static::lazy_static! {
    pub static ref ENGINE_CONFIG: EngineConfig = load_engine_config();
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EngineConfig {
    initialize: InitializeConfig,
    character: Character,
    volume: VolumeConfig,
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

    pub fn save_path(&self) -> &str {
        &self.initialize.save_path
    }

    pub fn main_volume(&self) -> f32 {
        self.volume.main
    }

    pub fn bgm_volume(&self) -> f32 {
        self.volume.bgm
    }

    pub fn voice_volume(&self) -> f32 {
        self.volume.voice
    }
}

fn load_engine_config() -> EngineConfig {
    let content = fs::read_to_string("./source/ini.toml").unwrap();
    toml::from_str(&content).unwrap()
}

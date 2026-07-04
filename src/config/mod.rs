use crate::config::initialize::{Character, InitializeConfig};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
};

pub mod figure;
pub mod initialize;
pub mod save_load;

pub mod cg;
pub mod character_volume;
pub mod extra;
pub mod system;
pub mod text;
pub mod user;
pub mod voice;
pub mod volume;

lazy_static::lazy_static! {
    pub static ref ENGINE_CONFIG: EngineConfig = load_engine_config();
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EngineConfig {
    initialize: InitializeConfig,
    character: Character,
}

impl EngineConfig {
    pub fn script_path(&self) -> &str {
        &self.initialize.script_path
    }

    pub fn background_path(&self) -> &str {
        &self.initialize.background_path
    }

    pub fn cg_path(&self) -> &str {
        &self.initialize.cg_path
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

    pub fn video_path(&self) -> &str {
        &self.initialize.video_path
    }

    pub fn video_extension(&self) -> &str {
        &self.initialize.video_extension
    }

    pub fn save_path(&self) -> &str {
        &self.initialize.save_path
    }

    pub fn character_name_list(&self) -> HashSet<&String> {
        self.character.name_list()
    }

    pub fn character_full_name_list(&self) -> HashSet<&String> {
        self.character.full_name_list()
    }

    pub fn character_list(&self) -> &HashMap<String, String> {
        self.character.list()
    }
}

fn load_engine_config() -> EngineConfig {
    let content = fs::read_to_string("./source/ini.toml").unwrap();
    toml::from_str(&content).unwrap()
}

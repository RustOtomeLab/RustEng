use crate::config::initialize::{Character, InitializeConfig};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
};

pub(crate) mod figure;
pub(crate) mod initialize;
pub(crate) mod save_load;

pub(crate) mod cg;
pub(crate) mod character_volume;
pub(crate) mod extra;
pub(crate) mod system;
pub(crate) mod text;
pub(crate) mod user;
pub(crate) mod voice;
pub(crate) mod volume;

lazy_static::lazy_static! {
    pub(crate) static ref ENGINE_CONFIG: EngineConfig = load_engine_config();
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct EngineConfig {
    initialize: InitializeConfig,
    character: Character,
}

impl EngineConfig {
    pub(crate) fn script_path(&self) -> &str {
        &self.initialize.script_path
    }

    pub(crate) fn background_path(&self) -> &str {
        &self.initialize.background_path
    }

    pub(crate) fn cg_path(&self) -> &str {
        &self.initialize.cg_path
    }

    pub(crate) fn voice_path(&self) -> &str {
        &self.initialize.voice_path
    }

    pub(crate) fn bgm_path(&self) -> &str {
        &self.initialize.bgm_path
    }

    pub(crate) fn figure_path(&self) -> &str {
        &self.initialize.figure_path
    }

    pub(crate) fn video_path(&self) -> &str {
        &self.initialize.video_path
    }

    pub(crate) fn video_extension(&self) -> &str {
        &self.initialize.video_extension
    }

    pub(crate) fn save_path(&self) -> &str {
        &self.initialize.save_path
    }

    pub(crate) fn character_name_list(&self) -> HashSet<&String> {
        self.character.name_list()
    }

    pub(crate) fn character_full_name_list(&self) -> HashSet<&String> {
        self.character.full_name_list()
    }

    pub(crate) fn character_list(&self) -> &HashMap<String, String> {
        self.character.list()
    }
}

fn load_engine_config() -> EngineConfig {
    let content = fs::read_to_string("./source/ini.toml").unwrap();
    toml::from_str(&content).unwrap()
}

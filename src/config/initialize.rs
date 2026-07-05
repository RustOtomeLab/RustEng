use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct InitializeConfig {
    pub(crate) script_path: String,
    pub(crate) background_path: String,
    pub(crate) cg_path: String,
    pub(crate) voice_path: String,
    pub(crate) bgm_path: String,
    pub(crate) figure_path: String,
    pub(crate) video_path: String,
    pub(crate) video_extension: String,
    pub(crate) save_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Character(HashMap<String, String>);

impl Character {
    pub(crate) fn list(&self) -> &HashMap<String, String> {
        &self.0
    }

    pub(crate) fn name_list(&self) -> HashSet<&String> {
        self.0.keys().collect()
    }

    pub(crate) fn full_name_list(&self) -> HashSet<&String> {
        self.0.values().collect()
    }
}

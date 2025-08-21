use std::collections::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct InitializeConfig {
    pub(crate) script_path: String,
    pub(crate) background_path: String,
    pub(crate) voice_path: String,
    pub(crate) bgm_path: String,
    pub(crate) figure_path: String,
    pub(crate) save_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Character {
    pub(crate) list: HashSet<String>,
}

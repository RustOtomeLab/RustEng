use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct InitializeConfig {
    pub(crate) script_path: String,
    pub(crate) background_path: String,
    pub(crate) cg_path: String,
    pub(crate) voice_path: String,
    pub(crate) bgm_path: String,
    pub(crate) figure_path: String,
    pub(crate) save_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Character {
    pub(crate) list: HashSet<String>,
}

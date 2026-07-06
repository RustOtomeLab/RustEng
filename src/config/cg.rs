use crate::config::ENGINE_CONFIG;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};

lazy_static::lazy_static! {
    pub(crate) static ref CG_LENGTH: CgLength = load_cg();
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CgConfig {
    cg: u64,
}

impl CgConfig {
    pub(crate) fn new(cg: u64) -> Self {
        Self { cg }
    }

    pub(crate) fn cg(&self) -> u64 {
        self.cg
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Length {
    name: String,
    index: usize,
    length: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct LengthWrapper {
    cast: Vec<Length>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CgLength {
    cg_by_name: HashMap<String, (usize, u64)>,
    cg_by_id: HashMap<usize, (String, u64)>,
}

impl CgLength {
    pub(crate) fn find_by_name(&self, name: &str) -> Option<&(usize, u64)> {
        self.cg_by_name.get(name)
    }

    pub(crate) fn find_by_id(&self, index: u64) -> Option<&(String, u64)> {
        self.cg_by_id.get(&(index as usize))
    }
}

fn load_cg() -> CgLength {
    let content = fs::read_to_string(format!("{}length.toml", ENGINE_CONFIG.cg_path(),)).unwrap();
    let name_item: LengthWrapper = toml::from_str(&content).unwrap();
    let index_item = name_item.cast.clone();
    CgLength {
        cg_by_name: name_item
            .cast
            .into_iter()
            .map(|length| (length.name, (length.index, length.length)))
            .collect(),
        cg_by_id: index_item
            .into_iter()
            .map(|length| (length.index, (length.name, length.length)))
            .collect(),
    }
}

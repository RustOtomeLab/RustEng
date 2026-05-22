use std::fs;
use serde::{Deserialize, Serialize};
use crate::config::cg::CgConfig;
use crate::config::ENGINE_CONFIG;
use crate::error::{EngineError, SaveError};
use crate::executor::executor::Executor;

lazy_static::lazy_static! {
    pub static ref EXTRA_CONFIG: ExtraConfig = load_extra_config();
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExtraConfig {
    cg: CgConfig,
}

impl ExtraConfig {
    pub fn cg(&self) -> u64 {
        self.cg.cg()
    }
}

impl Executor {
    pub fn load_extra(&mut self) {
        let mut cg = self.cg.borrow_mut();
        *cg = EXTRA_CONFIG.cg();
    }
}

fn load_extra_config() -> ExtraConfig {
    let content = fs::read_to_string(format!("{}/extra.toml", ENGINE_CONFIG.save_path())).unwrap();
    toml::from_str(&content).unwrap()
}

pub fn save_extra_config(cg: u64) -> Result<(), EngineError> {
    let path = format!("{}/extra.toml", ENGINE_CONFIG.save_path());
    let content = toml::to_string(&ExtraConfig{cg: CgConfig::new(cg)}).map_err(SaveError::from)?;
    fs::write(&path, content).map_err(|e| SaveError::Write {
        path,
        source: e,
    })?;

    Ok(())
}

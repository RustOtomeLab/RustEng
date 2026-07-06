use crate::config::{cg::CgConfig, ENGINE_CONFIG};
use crate::error::{EngineError, SaveError};
use crate::executors::executor::Executor;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ExtraConfig {
    cg: CgConfig,
}

impl ExtraConfig {
    pub(crate) fn cg(&self) -> u64 {
        self.cg.cg()
    }
}

impl Executor {
    pub(crate) fn load_extra(&mut self) {
        let extra_config = load_extra_config();
        self.set_cg(extra_config.cg());
    }
}

fn load_extra_config() -> ExtraConfig {
    let content = fs::read_to_string(format!("{}/extra.toml", ENGINE_CONFIG.save_path())).unwrap();
    toml::from_str(&content).unwrap()
}

pub(crate) fn save_extra_config(cg: u64) -> Result<(), EngineError> {
    let path = format!("{}/extra.toml", ENGINE_CONFIG.save_path());
    let content = toml::to_string(&ExtraConfig {
        cg: CgConfig::new(cg),
    })
    .map_err(SaveError::from)?;
    fs::write(&path, content).map_err(|e| SaveError::Write { path, source: e })?;

    Ok(())
}

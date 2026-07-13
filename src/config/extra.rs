use crate::config::cg::CG_CONFIG;
use crate::config::{cg::CgMap, ENGINE_CONFIG};
use crate::error::{EngineError, SaveError};
use crate::executors::executor::Executor;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ExtraConfig {
    cg: CgMap,
}

impl ExtraConfig {
    pub(crate) fn cg(self) -> Vec<u64> {
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
    if let Ok(content) = fs::read_to_string(format!("{}/extra.toml", ENGINE_CONFIG.save_path())) {
        toml::from_str(&content).unwrap()
    } else {
        let num = CG_CONFIG.length() / 64 + 1;
        ExtraConfig {
            cg: CgMap::new(vec![0; num]),
        }
    }
}

pub(crate) fn save_extra_config(cg: Rc<RefCell<Vec<u64>>>) -> Result<(), EngineError> {
    let cg = cg.borrow();
    let path = format!("{}/extra.toml", ENGINE_CONFIG.save_path());
    let content = toml::to_string(&ExtraConfig {
        cg: CgMap::new(cg.clone()),
    })
    .map_err(SaveError::from)?;
    fs::write(&path, content).map_err(|e| SaveError::Write { path, source: e })?;

    Ok(())
}

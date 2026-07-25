use crate::config::{cg::CG_CONFIG, ENGINE_CONFIG};
use crate::error::{EngineError, SaveError};
use crate::executors::executor::Executor;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;
use crate::data::cg::CgMap;

lazy_static::lazy_static! {
    pub(crate) static ref USER_DATA: UserData = load_user_data();
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct UserData {
    cg: CgMap,
}

impl UserData {
    pub(crate) fn cg(&self) -> Vec<u64> {
        self.cg.cg()
    }
}

impl Executor {
    pub(crate) fn load_cg(&mut self) {
        self.set_cg(USER_DATA.cg());
    }
}

fn load_user_data() -> UserData {
    if let Ok(content) = fs::read_to_string(format!("{}/data.toml", ENGINE_CONFIG.save_path())) {
        toml::from_str(&content).unwrap()
    } else {
        let num = CG_CONFIG.length() / 64 + 1;
        UserData {
            cg: CgMap::new(vec![0; num]),
        }
    }
}

pub(crate) fn save_user_data(cg: Rc<RefCell<Vec<u64>>>) -> Result<(), EngineError> {
    let cg = cg.borrow();
    let path = format!("{}/data.toml", ENGINE_CONFIG.save_path());
    let content = toml::to_string(&UserData {
        cg: CgMap::new(cg.clone()),
    })
        .map_err(SaveError::from)?;
    fs::write(&path, content).map_err(|e| SaveError::Write { path, source: e })?;

    Ok(())
}

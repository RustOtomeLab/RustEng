use crate::config::ENGINE_CONFIG;
use crate::error::{EngineError, SaveError};
use crate::executor::executor::Executor;
use serde::{Deserialize, Serialize};
use slint::{Image, ToSharedString, VecModel};
use std::fs;
use std::path::Path;
use std::rc::Rc;

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveData {
    pub(crate) script: String,
    pub(crate) block_index: usize,
    pub(crate) explain: String,
    pub(crate) image_path: String,
}

impl SaveData {
    pub fn new(
        script: String,
        block_index: usize,
        explain: String,
        image_path: String,
    ) -> SaveData {
        SaveData {
            script,
            block_index,
            explain,
            image_path,
        }
    }
}

impl Executor {
    pub fn load_save_data(&mut self) -> Result<(), EngineError> {
        let mut load_items = Vec::with_capacity(16);
        for i in 0..16 {
            let path = format!("{}{}.toml", ENGINE_CONFIG.save_path(), i);
            if let Ok(content) = fs::read_to_string(&path) {
                let SaveData {
                    script,
                    block_index,
                    explain,
                    image_path,
                } = toml::from_str(&content).map_err(|e| SaveError::Deserialize {
                    path: path.clone(),
                    source: e,
                })?;
                let image =
                    Image::load_from_path(Path::new(&image_path)).unwrap_or(Image::default());
                load_items.push((
                    image,
                    explain.to_shared_string(),
                    block_index as i32,
                    script.to_shared_string(),
                ));
            } else {
                let sava_data =
                    SaveData::new("".to_string(), 0, "空的".to_string(), "".to_string());
                let content = toml::to_string_pretty(&sava_data).map_err(SaveError::from)?;
                load_items.push((
                    Image::default(),
                    sava_data.explain.to_shared_string(),
                    sava_data.block_index as i32,
                    sava_data.script.to_shared_string(),
                ));
                fs::write(&path, content).map_err(|e| SaveError::Write {
                    path: path.clone(),
                    source: e,
                })?;
            }
        }

        let weak = self.get_weak();
        if let Some(window) = weak.upgrade() {
            window.set_save_items(Rc::new(VecModel::from(load_items)).into());
        }
        Ok(())
    }
}

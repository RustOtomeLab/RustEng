use std::fs;
use std::path::Path;
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use slint::{Image, ToSharedString, VecModel};
use crate::config::ENGINE_CONFIG;
use crate::error::EngineError;
use crate::executor::executor::Executor;

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveData {
    pub(crate) script: String,
    pub(crate) block_index: usize,
    pub(crate) explain: String,
    pub(crate) image_path: String,
}

impl SaveData {
   pub fn new(script: String, block_index: usize, explain: String, image_path: String) -> SaveData {
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
            if let Ok(content) = fs::read_to_string(format!("{}{}.toml", ENGINE_CONFIG.save_path(), i)) {
                let SaveData { script, block_index, explain, image_path } = toml::from_str(&content)?;
                let image = Image::load_from_path(Path::new(&image_path))
                    .unwrap_or(Image::default());
                load_items.push((image, explain.to_shared_string(), block_index as i32, script.to_shared_string()));
            }
        }

        let weak = self.get_weak();
        if let Some(window) = weak.upgrade() {
            window.set_save_items(Rc::new(VecModel::from(load_items)).into());
        }
        Ok(())
    }
}

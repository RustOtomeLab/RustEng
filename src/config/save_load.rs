use crate::config::ENGINE_CONFIG;
use crate::error::{EngineError, SaveError};
use crate::executors::executor::Executor;
use crate::ui::initialize::SaveItem;
use serde::{Deserialize, Serialize};
use slint::{Image, ModelRc, ToSharedString, VecModel};
use std::{fs, path::Path, rc::Rc};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct SaveData {
    pub(crate) script: String,
    pub(crate) block_index: usize,
    pub(crate) explain: String,
    pub(crate) image_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct SaveDataWrapper {
    save_data: Vec<SaveData>,
}

impl SaveData {
    pub(crate) fn new(
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
    pub(crate) fn load_save_data(&mut self) -> Result<(), EngineError> {
        let mut load_items: Vec<ModelRc<SaveItem>> = Vec::with_capacity(10);
        for i in 0..10 {
            let path = format!("{}{}.toml", ENGINE_CONFIG.save_path(), i);
            if let Ok(content) = fs::read_to_string(&path) {
                let wrapper: SaveDataWrapper =
                    toml::from_str(&content).map_err(|e| SaveError::Deserialize {
                        path: path.clone(),
                        source: e,
                    })?;
                let mut load_page = Vec::with_capacity(16);
                for SaveData {
                    script,
                    block_index,
                    explain,
                    image_path,
                } in wrapper.save_data
                {
                    let image = Image::load_from_path(Path::new(&image_path)).unwrap_or_default();
                    load_page.push(SaveItem {
                        bg: image,
                        explain: explain.to_shared_string(),
                        index: block_index as i32,
                        name: script.to_shared_string(),
                    });
                }
                load_items.push(Rc::new(VecModel::from(load_page)).into());
            } else {
                let wrapper = SaveDataWrapper {
                    save_data: vec![
                        SaveData::new(
                            "".to_string(),
                            0,
                            "空的".to_string(),
                            "".to_string()
                        );
                        16
                    ],
                };
                let content = toml::to_string_pretty(&wrapper).map_err(SaveError::from)?;
                load_items.push(
                    Rc::new(VecModel::from(vec![
                        SaveItem {
                            bg: Image::default(),
                            explain: "空的".to_shared_string(),
                            index: 0,
                            name: "".to_shared_string(),
                        };
                        16
                    ]))
                    .into(),
                );
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

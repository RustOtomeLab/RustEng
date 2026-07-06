use crate::config::{user::USER_CONFIG, ENGINE_CONFIG};
use crate::executors::executor::Executor;
use crate::ui::initialize::{CharacterVolume, MainWindow};
use serde::{Deserialize, Serialize};
use slint::{Model, Weak};
use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CharacterVolumeConfig {
    #[serde(flatten)]
    pub(crate) volumes: HashMap<String, f32>,
}

impl CharacterVolumeConfig {
    pub(crate) fn default_from_engine() -> Self {
        CharacterVolumeConfig {
            volumes: ENGINE_CONFIG
                .character_full_name_list()
                .iter()
                .map(|name| (name.to_string(), 100.0_f32))
                .collect(),
        }
    }

    pub(crate) fn fill_missing(&mut self) {
        for name in ENGINE_CONFIG.character_full_name_list() {
            self.volumes.entry(name.clone()).or_insert(100.0);
        }
    }

    pub(crate) fn from_weak(weak: Weak<MainWindow>) -> Self {
        if let Some(window) = weak.upgrade() {
            let model = window.get_character_volumes();
            let volumes = (0..model.row_count())
                .filter_map(|i| model.row_data(i))
                .map(|item| (item.name.to_string(), item.volume))
                .collect();
            CharacterVolumeConfig { volumes }
        } else {
            unreachable!()
        }
    }
}

impl Executor {
    pub(crate) fn load_character_volumes(&self) {
        let weak = self.get_weak();
        if let Some(window) = weak.upgrade() {
            let volumes: Vec<CharacterVolume> = ENGINE_CONFIG
                .character_full_name_list()
                .iter()
                .map(|name| CharacterVolume {
                    name: name.as_str().into(),
                    volume: USER_CONFIG.character_volume(name),
                })
                .collect();

            window.set_character_volumes(Rc::new(slint::VecModel::from(volumes)).into());
        }
    }
}

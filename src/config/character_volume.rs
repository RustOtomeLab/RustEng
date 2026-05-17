use std::collections::HashMap;
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use slint::{Model, Weak};
use crate::config::ENGINE_CONFIG;
use crate::config::user::USER_CONFIG;
use crate::executor::executor::Executor;
use crate::ui::ui::{CharacterVolume, MainWindow};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CharacterVolumeConfig {
    #[serde(flatten)]
    pub(crate) volumes: HashMap<String, f32>,
}

impl CharacterVolumeConfig {
    pub(crate) fn default_from_engine() -> Self {
        CharacterVolumeConfig {
            volumes: ENGINE_CONFIG
                .characters()
                .iter()
                .map(|name| (name.clone(), 100.0_f32))
                .collect(),
        }
    }

    pub(crate) fn fill_missing(&mut self) {
        for name in ENGINE_CONFIG.characters() {
            self.volumes.entry(name.clone()).or_insert(100.0);
        }
    }

    pub fn from_weak(weak: Weak<MainWindow>) -> Self {
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
    pub fn load_character_volumes(&self) {
        let weak = self.get_weak();
        if let Some(window) = weak.upgrade() {
            let volumes: Vec<CharacterVolume> = ENGINE_CONFIG
                .characters()
                .iter()
                .map(|name| CharacterVolume {
                    name: name.as_str().into(),
                    volume: USER_CONFIG.character_volume(name),
                })
                .collect();

            window.set_character_volumes(
                Rc::new(slint::VecModel::from(volumes)).into()
            );
        }
    }
}

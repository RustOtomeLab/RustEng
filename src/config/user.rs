use crate::config::{
    character_volume::CharacterVolumeConfig, system::AutoConfig, text::TextConfig,
    volume::VolumeConfig, ENGINE_CONFIG,
};
use crate::error::{EngineError, SaveError};
use crate::ui::initialize::MainWindow;
use serde::{Deserialize, Serialize};
use slint::Weak;
use std::fs;

lazy_static::lazy_static! {
    pub static ref USER_CONFIG: UserConfig = load_user_config();
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserConfig {
    auto: AutoConfig,
    text: TextConfig,
    volume: VolumeConfig,
    character_volume: CharacterVolumeConfig,
}

impl UserConfig {
    fn default() -> Self {
        UserConfig {
            auto: AutoConfig::default(),
            text: TextConfig::default(),
            volume: VolumeConfig::default(),
            character_volume: CharacterVolumeConfig::default_from_engine(),
        }
    }

    pub fn delay(&self) -> f32 {
        self.auto.delay()
    }

    pub fn is_wait(&self) -> bool {
        self.auto.is_wait()
    }

    pub fn main_volume(&self) -> f32 {
        self.volume.main()
    }

    pub fn bgm_volume(&self) -> f32 {
        self.volume.bgm()
    }

    pub fn voice_volume(&self) -> f32 {
        self.volume.voice()
    }

    pub fn speed(&self) -> f32 {
        self.text.speed()
    }

    pub fn opacity(&self) -> f32 {
        self.text.opacity()
    }

    pub fn character_volume(&self, name: &str) -> f32 {
        *self.character_volume.volumes.get(name).unwrap()
    }

    pub fn from_weak(weak: Weak<MainWindow>) -> Self {
        UserConfig {
            auto: AutoConfig::from_weak(weak.clone()),
            text: TextConfig::from_weak(weak.clone()),
            volume: VolumeConfig::from_weak(weak.clone()),
            character_volume: CharacterVolumeConfig::from_weak(weak),
        }
    }
}

fn load_user_config() -> UserConfig {
    let path = format!("{}/user.toml", ENGINE_CONFIG.save_path());

    match fs::read_to_string(&path) {
        Ok(content) => match toml::from_str::<UserConfig>(&content) {
            Ok(mut config) => {
                config.character_volume.fill_missing();
                config
            }
            Err(_) => {
                let config = UserConfig::default();
                let _ = write_config(&path, &config);
                config
            }
        },
        Err(_) => {
            let config = UserConfig::default();
            let _ = write_config(&path, &config);
            config
        }
    }
}

pub fn save_user_config(weak: Weak<MainWindow>) -> Result<(), EngineError> {
    let path = format!("{}/user.toml", ENGINE_CONFIG.save_path());
    write_config(&path, &UserConfig::from_weak(weak))?;

    Ok(())
}

fn write_config(path: &str, config: &UserConfig) -> Result<(), EngineError> {
    let content = toml::to_string(config).map_err(SaveError::from)?;
    fs::write(path, content).map_err(|e| SaveError::Write {
        path: path.to_string(),
        source: e,
    })?;
    Ok(())
}

use crate::config::system::AutoConfig;
use crate::config::volume::VolumeConfig;
use crate::config::ENGINE_CONFIG;
use crate::error::EngineError;
use crate::ui::ui::MainWindow;
use serde::{Deserialize, Serialize};
use slint::Weak;
use std::fs;

lazy_static::lazy_static! {
    pub static ref USER_CONFIG: UserConfig = load_user_config();
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserConfig {
    auto: AutoConfig,
    volume: VolumeConfig,
}

impl UserConfig {
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

    pub fn from_weak(weak: Weak<MainWindow>) -> Self {
        UserConfig {
            auto: AutoConfig::from_weak(weak.clone()),
            volume: VolumeConfig::from_weak(weak),
        }
    }
}

fn load_user_config() -> UserConfig {
    let content = fs::read_to_string(format!("{}/user.toml", ENGINE_CONFIG.save_path())).unwrap();
    toml::from_str(&content).unwrap()
}

pub fn save_user_config(weak: Weak<MainWindow>) -> Result<(), EngineError> {
    fs::write(
        format!("{}/user.toml", ENGINE_CONFIG.save_path()),
        toml::to_string(&UserConfig::from_weak(weak))?,
    )?;

    Ok(())
}

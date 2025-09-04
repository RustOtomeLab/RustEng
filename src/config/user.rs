use crate::config::script::AutoConfig;
use crate::config::volume::VolumeConfig;
use crate::config::ENGINE_CONFIG;
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::Duration;

lazy_static::lazy_static! {
    pub static ref USER_CONFIG: UserConfig = load_user_config();
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserConfig {
    auto: AutoConfig,
    volume: VolumeConfig,
}

impl UserConfig {
    pub fn delay(&self) -> Duration {
        self.auto.delay()
    }

    pub fn is_wait(&self) -> bool {
        self.auto.is_wait()
    }

    pub fn main_volume(&self) -> f32 {
        self.volume.main
    }

    pub fn bgm_volume(&self) -> f32 {
        self.volume.bgm
    }

    pub fn voice_volume(&self) -> f32 {
        self.volume.voice
    }

    pub fn volume(&mut self, main: f32, bgm: f32, voice: f32) {
        self.volume = VolumeConfig { main, bgm, voice };
    }
}

fn load_user_config() -> UserConfig {
    let content = fs::read_to_string(format!("{}/user.toml", ENGINE_CONFIG.save_path())).unwrap();
    toml::from_str(&content).unwrap()
}

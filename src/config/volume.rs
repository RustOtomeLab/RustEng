use crate::config::{EngineConfig, ENGINE_CONFIG};
use crate::error::EngineError;
use crate::executor::executor::Executor;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct VolumeConfig {
    pub(crate) main: f32,
    pub(crate) bgm: f32,
    pub(crate) voice: f32,
}

pub fn save_volume(main: f32, bgm: f32, voice: f32) -> Result<(), EngineError> {
    let content = fs::read_to_string("./source/ini.toml")?;
    let mut config: EngineConfig = toml::from_str(&content)?;
    config.volume = VolumeConfig { main, bgm, voice };
    fs::write("./source/ini.toml", toml::to_string(&config)?)?;

    Ok(())
}

impl Executor {
    pub fn load_volume(&self) -> Result<(), EngineError> {
        let weak = self.get_weak();
        if let Some(window) = weak.upgrade() {
            window.set_main_volume(ENGINE_CONFIG.main_volume());
            window.set_bgm_volume(ENGINE_CONFIG.bgm_volume());
            window.set_voice_volume(ENGINE_CONFIG.voice_volume());
        }

        Ok(())
    }
}

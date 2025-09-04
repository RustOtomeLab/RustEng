use crate::config::user::USER_CONFIG;
use crate::executor::executor::Executor;
use crate::ui::ui::MainWindow;
use serde::{Deserialize, Serialize};
use slint::Weak;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct VolumeConfig {
    main: f32,
    bgm: f32,
    voice: f32,
}

impl VolumeConfig {
    pub fn main(&self) -> f32 {
        self.main
    }

    pub fn bgm(&self) -> f32 {
        self.bgm
    }

    pub fn voice(&self) -> f32 {
        self.voice
    }

    pub fn from_weak(weak: Weak<MainWindow>) -> Self {
        if let Some(window) = weak.upgrade() {
            VolumeConfig {
                main: window.get_main_volume(),
                bgm: window.get_bgm_volume(),
                voice: window.get_voice_volume(),
            }
        } else {
            unreachable!()
        }
    }
}

impl Executor {
    pub fn load_volume(&self) {
        let weak = self.get_weak();
        if let Some(window) = weak.upgrade() {
            window.set_main_volume(USER_CONFIG.main_volume());
            window.set_bgm_volume(USER_CONFIG.bgm_volume());
            window.set_voice_volume(USER_CONFIG.voice_volume());
        }
    }
}

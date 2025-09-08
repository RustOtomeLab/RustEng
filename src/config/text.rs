use crate::config::user::USER_CONFIG;
use crate::executor::executor::Executor;
use crate::ui::ui::MainWindow;
use serde::{Deserialize, Serialize};
use slint::Weak;

#[derive(Debug, Deserialize, Serialize)]
pub struct TextConfig {
    speed: f32,
    opacity: f32,
}

impl TextConfig {
    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    pub fn from_weak(weak: Weak<MainWindow>) -> Self {
        if let Some(window) = weak.upgrade() {
            TextConfig {
                speed: window.get_text_speed(),
                opacity: window.get_dialogue_opacity(),
            }
        } else {
            unreachable!()
        }
    }
}

impl Executor {
    pub fn load_text(&mut self) {
        let weak = self.get_weak();
        if let Some(window) = weak.upgrade() {
            window.set_text_speed(USER_CONFIG.speed());
            window.set_dialogue_opacity(USER_CONFIG.opacity());
        }
    }
}

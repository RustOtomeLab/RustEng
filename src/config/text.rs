use crate::config::user::USER_CONFIG;
use crate::executors::executor::Executor;
use crate::ui::initialize::MainWindow;
use serde::{Deserialize, Serialize};
use slint::Weak;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct TextConfig {
    speed: f32,
    opacity: f32,
}

impl Default for TextConfig {
    fn default() -> Self {
        TextConfig {
            speed: 50.0,
            opacity: 0.8,
        }
    }
}
impl TextConfig {
    pub(crate) fn speed(&self) -> f32 {
        self.speed
    }

    pub(crate) fn opacity(&self) -> f32 {
        self.opacity
    }

    pub(crate) fn from_weak(weak: Weak<MainWindow>) -> Self {
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
    pub(crate) fn load_text(&mut self) {
        let weak = self.get_weak();
        if let Some(window) = weak.upgrade() {
            window.set_text_speed(USER_CONFIG.speed());
            window.set_dialogue_opacity(USER_CONFIG.opacity());
        }
    }
}

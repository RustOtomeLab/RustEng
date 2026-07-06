use crate::config::user::USER_CONFIG;
use crate::executors::executor::Executor;
use crate::ui::initialize::MainWindow;
use serde::{Deserialize, Serialize};
use slint::Weak;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct AutoConfig {
    delay: f32,
    is_wait: bool,
}

impl Default for AutoConfig {
    fn default() -> Self {
        AutoConfig {
            delay: 3.5,
            is_wait: true,
        }
    }
}

impl AutoConfig {
    pub(crate) fn delay(&self) -> f32 {
        self.delay
    }

    pub(crate) fn is_wait(&self) -> bool {
        self.is_wait
    }

    pub(crate) fn from_weak(weak: Weak<MainWindow>) -> Self {
        if let Some(window) = weak.upgrade() {
            AutoConfig {
                delay: window.get_delay(),
                is_wait: window.get_is_wait(),
            }
        } else {
            unreachable!()
        }
    }
}

impl Executor {
    pub(crate) fn load_auto(&mut self) {
        let weak = self.get_weak();
        if let Some(window) = weak.upgrade() {
            window.set_is_wait(USER_CONFIG.is_wait());
            window.set_delay(USER_CONFIG.delay());
        }
    }
}

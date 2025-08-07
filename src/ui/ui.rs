use crate::audio::player::{BgmPlayer, play_voice};
use crate::executor::script_executor::execute_script;
use crate::script::Script;
use crate::error::EngineError;

use slint::{Image, SharedString};
use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;

slint::include_modules!();

#[derive(Debug, Clone, Default)]
pub struct UiRenderBlock {
    pub dialogue: Option<(String, String)>,
    pub background: Option<String>,
    pub bgm: Option<String>,
    pub voice: Option<String>,
}

static BACKGROUND_PATH: &str = "./background/";
static VOICE_PATH: &str = "./voice/";
static BGM_PATH: &str = "./bgm/";

pub async fn ui(script: Rc<RefCell<Script>>, bgm_player: Rc<RefCell<BgmPlayer>>) -> Result<(), EngineError> {
    let window = MainWindow::new()?;
    let weak = window.as_weak();

    window.on_clicked({
        let script = script.clone();
        let bgm_player = bgm_player.clone();
        let weak = weak.clone();
        move || {
            let script = script.clone();
            let bgm_player = bgm_player.clone();
            if let Some(window) = weak.upgrade() {
                slint::spawn_local(async move {
                    let mut script = script.borrow_mut();
                    let mut bgm_player = bgm_player.borrow_mut();
                    if let Some(block) = execute_script(&mut script) {
                        if let Some(voice) = block.voice {
                            play_voice(&format!("{}{}", VOICE_PATH, voice)).await;
                        }

                        if let Some((speaker, text)) = block.dialogue {
                            window.set_speaker(SharedString::from(speaker));
                            window.set_dialogue(SharedString::from(text));
                        }

                        if let Some(bg) = block.background {
                            let image = Image::load_from_path(Path::new(&format!("{}{}", BACKGROUND_PATH, bg))).unwrap();
                            window.set_bg(image);
                        }

                        if let Some(bgm) = block.bgm {
                            bgm_player.play_loop(&format!("{}{}", BGM_PATH, bgm));
                        }
                    }
                }).expect("TODO: panic message");
            }
        }
    });

    window.run()?;
    Ok(())
}
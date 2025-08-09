use crate::audio::player::Player;
use crate::error::EngineError;
use crate::executor::script_executor::execute_script;
use crate::script::Script;

use slint::{Image, SharedString};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;

slint::include_modules!();

#[derive(Debug, Clone, Default)]
pub struct UiRenderBlock {
    pub dialogue: Option<(String, String)>,
    pub background: Option<String>,
    pub bgm: Option<String>,
    pub voice: Option<String>,
    pub figure: Option<(String, String, String, String, String)>,
}

static BACKGROUND_PATH: &str = "./source/background/";
static VOICE_PATH: &str = "./source/voice/";
static BGM_PATH: &str = "./source/bgm/";
static FG_PATH: &str = "./source/figure/";

pub async fn ui(
    script: Rc<RefCell<Script>>,
    bgm_player: Rc<RefCell<Player>>,
    voice_player: Rc<RefCell<Player>>,
) -> Result<(), EngineError> {
    let window = MainWindow::new()?;
    let weak = window.as_weak();

    let weak_for_fullscreen = weak.clone();
    let mut is_fullscreen = false;
    window.on_toggle_fullscreen(move || {
        if let Some(window) = weak_for_fullscreen.upgrade() {
            is_fullscreen = !is_fullscreen;
            if is_fullscreen {
                window.window().set_fullscreen(true);
                window.set_is_fullscreen(true);
            } else {
                window.window().set_fullscreen(false);
                window.set_is_fullscreen(false);
            }
        }
    });

    let weak_for_volume = weak.clone();
    window.on_volume_changed({
        let bgm_player = bgm_player.clone();
        let voice_player = voice_player.clone();
        move || {
            let bgm_player = bgm_player.clone();
            let voice_player = voice_player.clone();
            if let Some(window) = weak_for_volume.upgrade() {
                slint::spawn_local(async move {
                    let mut bgm_player = bgm_player.borrow_mut();
                    let mut voice_player = voice_player.borrow_mut();
                    let volume = window.get_volume() as f32 / 100.0;
                    println!("Volume: {}", volume);
                    bgm_player.change_volume(volume);
                    voice_player.change_volume(volume);
                })
                    .expect("TODO: panic message");
            }
        }
    });

    window.on_clicked({
        let weak = weak.clone();
        move || {
            //let time = Instant::now();
            let script = script.clone();
            let bgm_player = bgm_player.clone();
            let voice_player = voice_player.clone();
            if let Some(window) = weak.upgrade() {
                slint::spawn_local(async move {
                    let mut script = script.borrow_mut();
                    let mut bgm_player = bgm_player.borrow_mut();
                    let mut voice_player = voice_player.borrow_mut();
                    if let Some(block) = execute_script(&mut script) {
                        if let Some(bg) = block.background {
                            let image = Image::load_from_path(Path::new(&format!(
                                "{}{}",
                                BACKGROUND_PATH, bg
                            )))
                            .unwrap();
                            window.set_bg(image);
                            //println!("{:?}", time.elapsed());
                        }

                        if let Some((name, distant, body, face, position)) = block.figure {
                            let body = Image::load_from_path(Path::new(&format!(
                                "{}{}/{}/{}.png",
                                FG_PATH, name, distant, body
                            ))).unwrap();
                            let face = Image::load_from_path(Path::new(&format!(
                                "{}{}/{}/{}.png",
                                FG_PATH, name, distant, face
                            ))).unwrap();
                            match &position[..] {
                                "0" => {
                                    window.set_fg_body_0(body);
                                    window.set_fg_face_0(face);
                                }
                                _ => ()
                            }
                        }

                        if let Some(voice) = block.voice {
                            let volume = window.get_volume() as f32 / 100.0;
                            voice_player.play_voice(&format!("{}{}", VOICE_PATH, voice), volume);
                            //println!("{:?}", time.elapsed());
                        }

                        if let Some((speaker, text)) = block.dialogue {
                            window.set_speaker(SharedString::from(speaker));
                            window.set_dialogue(SharedString::from(text));
                            //println!("{:?}", time.elapsed());
                        }

                        if let Some(bgm) = block.bgm {
                            let volume = window.get_volume() as f32 / 100.0;
                            bgm_player.play_loop(&format!("{}{}", BGM_PATH, bgm), volume);
                            //println!("{:?}", time.elapsed());
                        }
                    }
                })
                .expect("TODO: panic message");
            }
        }
    });

    window.run()?;
    Ok(())
}

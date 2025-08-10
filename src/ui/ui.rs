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
                    let volume = window.get_main_volume() / 100.0;
                    let bgm_volume = window.get_bgm_volume() / 100.0;
                    let voice_volume = window.get_voice_volume() / 100.0;
                    bgm_player.change_volume(volume * bgm_volume);
                    voice_player.change_volume(volume * voice_volume);
                })
                .expect("TODO: panic message");
            }
        }
    });

    let weak_for_bgm_volume = weak.clone();
    window.on_bgm_volume_changed({
        let bgm_player = bgm_player.clone();
        move || {
            let bgm_player = bgm_player.clone();
            if let Some(window) = weak_for_bgm_volume.upgrade() {
                slint::spawn_local(async move {
                    let mut bgm_player = bgm_player.borrow_mut();
                    let volume = window.get_main_volume() / 100.0;
                    let bgm_volume = window.get_bgm_volume() / 100.0;
                    bgm_player.change_volume(volume * bgm_volume);
                })
                .expect("TODO: panic message");
            }
        }
    });

    let weak_for_voice_volume = weak.clone();
    window.on_voice_volume_changed({
        let voice_player = voice_player.clone();
        move || {
            let voice_player = voice_player.clone();
            if let Some(window) = weak_for_voice_volume.upgrade() {
                slint::spawn_local(async move {
                    let mut voice_player = voice_player.borrow_mut();
                    let volume = window.get_main_volume() / 100.0;
                    let voice_volume = window.get_voice_volume() / 100.0;
                    voice_player.change_volume(volume * voice_volume);
                })
                .expect("TODO: panic message");
            }
        }
    });

    window.on_clicked({
        move || {
            let script = script.clone();
            let bgm_player = bgm_player.clone();
            let voice_player = voice_player.clone();
            let weak = weak.clone();
            slint::spawn_local(async move {
                execute_script(script, bgm_player, voice_player, weak).await
            })
            .expect("TODO: panic message");
        }
    });

    window.run()?;
    Ok(())
}

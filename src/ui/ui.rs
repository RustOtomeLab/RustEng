use crate::audio::player::Player;
use crate::error::EngineError;
use crate::executor::script_executor::{execute_bgm_volume, execute_choose, execute_save, execute_load, execute_script, execute_voice_volume};
use crate::script::Script;
use std::cell::RefCell;
use std::rc::Rc;

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
    
    let weak_for_save = weak.clone();
    window.on_save({ 
        let script = script.clone();
        move |index| {
            let script = script.clone();
            let weak = weak_for_save.clone();
            slint::spawn_local(async move {
                execute_save(script, index, weak).await
            })
            .expect("TODO: panic message");  
        }
    });

    let weak_for_load = weak.clone();
    window.on_load({
        let script = script.clone();
        let bgm_player = bgm_player.clone();
        let voice_player = voice_player.clone();
        move |name, index| {
            let script = script.clone();
            let bgm_player = bgm_player.clone();
            let voice_player = voice_player.clone();
            let weak = weak_for_load.clone();
            slint::spawn_local(async move {
                execute_load(script, name.to_string(), index, bgm_player, voice_player, weak).await
            })
            .expect("TODO: panic message");
        }
    });

    let weak_for_volume = weak.clone();
    window.on_volume_changed({
        let bgm_player = bgm_player.clone();
        let voice_player = voice_player.clone();
        move || {
            let weak_for_bgm = weak_for_volume.clone();
            let weak_for_voice = weak_for_volume.clone();
            let bgm_player = bgm_player.clone();
            let voice_player = voice_player.clone();
            slint::spawn_local(async move {
                let _ = execute_bgm_volume(bgm_player, weak_for_bgm).await;
                execute_voice_volume(voice_player, weak_for_voice).await
            })
            .expect("TODO: panic message");
        }
    });

    let weak_for_bgm_volume = weak.clone();
    window.on_bgm_volume_changed({
        let bgm_player = bgm_player.clone();
        move || {
            let weak = weak_for_bgm_volume.clone();
            let bgm_player = bgm_player.clone();
            slint::spawn_local(async move { execute_bgm_volume(bgm_player, weak).await })
                .expect("TODO: panic message");
        }
    });

    let weak_for_voice_volume = weak.clone();
    window.on_voice_volume_changed({
        let voice_player = voice_player.clone();
        move || {
            let weak = weak_for_voice_volume.clone();
            let voice_player = voice_player.clone();
            slint::spawn_local(async move { execute_voice_volume(voice_player, weak).await })
                .expect("TODO: panic message");
        }
    });

    let weak_for_choose = weak.clone();
    window.on_choose({
        let script = script.clone();
        let bgm_player = bgm_player.clone();
        move |choice| {
            let weak = weak_for_choose.clone();
            let script = script.clone();
            let bgm_player = bgm_player.clone();
            slint::spawn_local(
                async move { execute_choose(script, bgm_player, choice, weak).await },
            )
            .expect("TODO: panic message");
        }
    });

    let weak_for_click = weak.clone();
    window.on_clicked({
        move || {
            let script = script.clone();
            let bgm_player = bgm_player.clone();
            let voice_player = voice_player.clone();
            let weak = weak_for_click.clone();
            slint::spawn_local(async move {
                execute_script(script, bgm_player, voice_player, weak).await
            })
            .expect("TODO: panic message");
        }
    });

    window.on_exit({
        move || {
            slint::spawn_local({
                let weak = weak.clone();
                async move {
                    if let Some(window) = weak.upgrade() {
                        let _ = window.hide();
                    }
                    let _ = slint::quit_event_loop();
                }
            })
            .expect("TODO: panic message");
        }
    });

    window.run()?;
    Ok(())
}

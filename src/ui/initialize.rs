use crate::error::EngineError;
use crate::executors::{executor::Executor, load_data};
slint::include_modules!();

pub(crate) async fn ui() -> Result<(), EngineError> {
    let window = MainWindow::new()?;
    let weak = window.as_weak();

    let mut executor = Executor::new(weak)?;

    let executor_tx = load_data(&mut executor)?;

    let mut is_fullscreen = false;
    let weak_for_fullscreen = executor.get_weak();
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

    window.on_save({
        let mut executor = executor.clone();
        move |index, page_num| {
            executor
                .execute_save(index, page_num)
                .expect("Save panicked");
        }
    });

    window.on_load({
        let mut executor = executor.clone();
        move |name, index| {
            executor
                .execute_load(name.to_string(), index)
                .expect("Load panicked");
        }
    });

    window.on_get_ex({
        let executor = executor.clone();
        move || {
            executor.execute_get_ex().expect("Get Ex panicked");
        }
    });

    window.on_volume_changed({
        let mut executor = executor.clone();
        move || {
            executor
                .execute_bgm_volume()
                .expect("bgm_volume change panicked");
            executor
                .execute_voice_volume()
                .expect("voice_volume change panicked");
        }
    });

    window.on_bgm_volume_changed({
        let mut executor = executor.clone();
        move || {
            executor
                .execute_bgm_volume()
                .expect("Bgm volume change panicked");
        }
    });

    window.on_voice_volume_changed({
        let mut executor = executor.clone();
        move || {
            executor
                .execute_voice_volume()
                .expect("Voice volume change panicked");
        }
    });

    window.on_save_config({
        let executor = executor.clone();
        move || {
            executor.execute_save_config().expect("Choose panicked");
        }
    });

    window.on_choose({
        let mut executor = executor.clone();
        move |choice| {
            executor.execute_choose(choice).expect("Choose panicked");
        }
    });

    window.on_backlog({
        let executor = executor.clone();
        move || {
            executor.execute_backlog().expect("Backlog panicked");
        }
    });

    window.on_backlog_change({
        let mut executor = executor.clone();
        move |i| {
            executor
                .execute_backlog_change(i)
                .expect("Backlog change panicked");
        }
    });

    window.on_backlog_jump({
        let mut executor = executor.clone();
        move |name, i| {
            executor
                .execute_backlog_jump(name.to_string(), i)
                .expect("Backlog jump panicked");
        }
    });

    window.on_backlog_replay({
        let executor = executor.clone();
        move |name, voice| {
            executor
                .play_voice(&name.to_string(), &voice.to_string())
                .expect("Backlog resume panicked");
        }
    });

    window.on_replay_voice({
        let mut executor = executor.clone();
        move || {
            executor.execute_replay().expect("Backlog resume panicked");
        }
    });

    window.on_clicked({
        let mut executor = executor.clone();
        move || {
            executor.execute_script().expect("Clicked panicked");
        }
    });

    window.on_auto_play({
        let mut executor = executor.clone();
        let tx = executor_tx.auto_tx();
        move |source| {
            let tx = tx.clone();
            executor
                .execute_auto(tx, source)
                .expect("TODO: panic message");
        }
    });

    window.on_skip_play({
        let mut executor = executor.clone();
        let tx = executor_tx.skip_tx();
        move |source| {
            let tx = tx.clone();
            executor
                .execute_skip(tx, source)
                .expect("TODO: panic message");
        }
    });

    window.on_stop_video({
        let executor = executor.clone();
        move || {
            let executor = executor.clone();
            slint::spawn_local(async move {
                if let Err(e) = executor.execute_stop_video().await {
                    eprintln!("stop_video failed: {e}");
                }
            })
            .expect("stop_video panicked");
        }
    });

    window.on_exit({
        let weak = executor.get_weak();
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
            .expect("Exit panicked");
        }
    });

    window.run()?;
    Ok(())
}

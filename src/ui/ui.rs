use crate::audio::player::Player;
use crate::error::EngineError;
use crate::executor::auto_executor::AutoExecutor;
use crate::executor::executor::Executor;
use crate::script::Script;
use std::cell::RefCell;
use std::rc::Rc;
use crate::executor::delay_executor::DelayExecutor;

slint::include_modules!();

pub async fn ui(
    script: Rc<RefCell<Script>>,
    bgm_player: Rc<RefCell<Player>>,
    voice_player: Rc<RefCell<Player>>,
) -> Result<(), EngineError> {
    let window = MainWindow::new()?;
    let weak = window.as_weak();

    let mut executor = Executor::new(script, bgm_player, voice_player, weak);
    let (auto_executor, auto_tx) = AutoExecutor::new(executor.clone());
    let (delay_executor, delay_tx) = DelayExecutor::new(executor.clone());
    auto_executor.start_timer();
    delay_executor.start_timer();

    executor.load_save_data()?;

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
        let executor = executor.clone();
        move |index| {
            let mut executor = executor.clone();
            slint::spawn_local(async move { executor.execute_save(index).await })
                .expect("Save panicked");
        }
    });

    window.on_load({
        let executor = executor.clone();
        move |name, index| {
            let mut executor = executor.clone();
            slint::spawn_local(async move { executor.execute_load(name.to_string(), index).await })
                .expect("Load panicked");
        }
    });

    window.on_volume_changed({
        let executor = executor.clone();
        move || {
            let mut executor = executor.clone();
            slint::spawn_local(async move {
                let _ = executor.execute_bgm_volume().await;
                executor.execute_voice_volume().await
            })
            .expect("Volume change panicked");
        }
    });

    window.on_bgm_volume_changed({
        let executor = executor.clone();
        move || {
            let mut executor = executor.clone();
            slint::spawn_local(async move { executor.execute_bgm_volume().await })
                .expect("Bgm volume change panicked");
        }
    });

    window.on_voice_volume_changed({
        let executor = executor.clone();
        move || {
            let mut executor = executor.clone();
            slint::spawn_local(async move { executor.execute_voice_volume().await })
                .expect("Voice volume change panicked");
        }
    });

    window.on_choose({
        let executor = executor.clone();
        move |choice| {
            let mut executor = executor.clone();
            slint::spawn_local(async move { executor.execute_choose(choice).await })
                .expect("Choose panicked");
        }
    });

    window.on_backlog({
        let executor = executor.clone();
        move || {
            let mut executor = executor.clone();
            slint::spawn_local(async move { executor.execute_backlog().await })
                .expect("Backlog panicked");
        }
    });

    window.on_backlog_change({
        let executor = executor.clone();
        move |i| {
            let mut executor = executor.clone();
            slint::spawn_local(async move { executor.execute_backlog_change(i).await })
                .expect("Backlog change panicked");
        }
    });

    window.on_backlog_jump({
        let executor = executor.clone();
        move |name, i| {
            let mut executor = executor.clone();
            //println!("backlog {} {}", i, name);
            slint::spawn_local(
                async move { executor.execute_backlog_jump(name.to_string(), i).await },
            )
            .expect("Backlog jump panicked");
        }
    });

    window.on_clicked({
        let executor = executor.clone();
        move || {
            let tx = delay_tx.clone();
            let mut executor = executor.clone();
            slint::spawn_local(async move { executor.execute_script().await })
                .expect("Clicked panicked");
        }
    });

    window.on_auto_play({
        let executor = executor.clone();
        move |source| {
            let tx = auto_tx.clone();
            let mut executor = executor.clone();
            slint::spawn_local( async move {
                executor.execute_auto(tx, source).await
            })
            .expect("TODO: panic message");
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

use crate::audio::player::Player;
use crate::error::EngineError;
use crate::parser::script_parser::{Command, Commands};
use crate::script::{Label, Script};
use crate::ui::ui::MainWindow;
use slint::{Image, SharedString, VecModel, Weak};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

static BACKGROUND_PATH: &str = "./source/background/";
static VOICE_PATH: &str = "./source/voice/";
static BGM_PATH: &str = "./source/bgm/";
static FIGURE_PATH: &str = "./source/figure/";

pub async fn execute_bgm_volume(
    bgm_player: Rc<RefCell<Player>>,
    weak: Weak<MainWindow>,
) -> Result<(), EngineError> {
    if let Some(window) = weak.upgrade() {
        let bgm_player = bgm_player.borrow_mut();
        let volume = window.get_main_volume() / 100.0;
        let bgm_volume = window.get_bgm_volume() / 100.0;
        bgm_player.change_volume(volume * bgm_volume);
    }

    Ok(())
}

pub async fn execute_voice_volume(
    voice_player: Rc<RefCell<Player>>,
    weak: Weak<MainWindow>,
) -> Result<(), EngineError> {
    if let Some(window) = weak.upgrade() {
        let voice_player = voice_player.borrow_mut();
        let volume = window.get_main_volume() / 100.0;
        let voice_volume = window.get_voice_volume() / 100.0;
        voice_player.change_volume(volume * voice_volume);
    }

    Ok(())
}

pub async fn execute_choose(
    script: Rc<RefCell<Script>>,
    bgm_player: Rc<RefCell<Player>>,
    choice: SharedString,
    weak: Weak<MainWindow>,
) -> Result<(), EngineError> {
    let mut label = (String::default(), String::default());
    {
        let scr = script.clone();
        let scr = scr.borrow();
        label = scr.get_choice_label(&choice).unwrap().clone();
    }

    let mut volume = 0.0;
    let weak_for_jump = weak.clone();
    if let Some(window) = weak.upgrade() {
        window.set_choose_branch(Rc::new(VecModel::from(vec![])).into());
        window.set_current_choose(0);
        window.set_speaker("".into());
        window.set_dialogue(choice);
        volume = window.get_main_volume() * window.get_bgm_volume() / 10000.0;
    }

    execute_jump(script, bgm_player, volume, label, weak_for_jump).await
}

pub async fn execute_jump(
    script: Rc<RefCell<Script>>,
    bgm_player: Rc<RefCell<Player>>,
    volume: f32,
    label: Label,
    weak: Weak<MainWindow>,
) -> Result<(), EngineError> {
    let mut script = script.borrow_mut();
    let mut current_bgm = script.current_bgm().to_string();
    if label.0 != script.name() {
        *script = Script::from_name(label.0)?;
    }
    let mut current_block = script.index();
    if let Some(index) = script.find_label(&label.1) {
        current_block = *index;
        if let Some((_, bgm)) = script.get_bgm(*index) {
            if &current_bgm != bgm {
                let bgm_player = bgm_player.borrow_mut();
                bgm_player.play_loop(&format!("{}{}.ogg", BGM_PATH, bgm), volume);
                current_bgm = bgm.clone();
            }
        } else if let None = script.get_bgm(*index) {
            let bgm_player = bgm_player.borrow_mut();
            bgm_player.stop();
            current_bgm = String::new();
        }

        if let Some((_, background)) = script.get_background(*index) {
            if let Some(window) = weak.upgrade() {
                window.set_pre_bg(background.into());
            }
        }
    }
    script.set_current_bgm(current_bgm);
    script.set_index(current_block);

    Ok(())
}

pub async fn execute_script(
    script: Rc<RefCell<Script>>,
    bgm_player: Rc<RefCell<Player>>,
    voice_player: Rc<RefCell<Player>>,
    weak: Weak<MainWindow>,
) -> Result<(), EngineError> {
    let mut commands = Commands::EmptyCommands;
    {
        let scr = script.clone();
        let mut scr = scr.borrow_mut();
        if let Some(cmds) = scr.next_command() {
            commands = cmds.clone();
        }
    }
    match commands {
        Commands::EmptyCommands => unreachable!(),
        Commands::OneCommand(command) => {
            apply_command(command, script, bgm_player, voice_player, weak).await?;
        }
        Commands::VarCommands(vars) => {
            for command in vars {
                let script = script.clone();
                let bgm_player = bgm_player.clone();
                let voice_player = voice_player.clone();
                let weak = weak.clone();
                apply_command(command, script, bgm_player, voice_player, weak).await?;
            }
        }
    }
    Ok(())
}

async fn apply_command(
    command: Command,
    script: Rc<RefCell<Script>>,
    bgm_player: Rc<RefCell<Player>>,
    voice_player: Rc<RefCell<Player>>,
    weak: Weak<MainWindow>,
) -> Result<(), EngineError> {
    let weak_for_jump = weak.clone();
    if let Some(window) = weak.upgrade() {
        let pre_bg = window.get_pre_bg();
        if !pre_bg.is_empty() {
            let image =
                Image::load_from_path(Path::new(&format!("{}{}.png", BACKGROUND_PATH, pre_bg)))
                    .unwrap();
            window.set_bg(image);
            window.set_pre_bg("".into());
        }

        match command {
            Command::SetBackground(bg) => {
                let image =
                    Image::load_from_path(Path::new(&format!("{}{}.png", BACKGROUND_PATH, bg)))
                        .unwrap();
                window.set_bg(image);
            }
            Command::PlayBgm(bgm) => {
                let mut script = script.borrow_mut();
                if &bgm != script.current_bgm() {
                    script.set_current_bgm(bgm.clone());
                    let bgm_player = bgm_player.borrow_mut();
                    let volume = window.get_main_volume() / 100.0;
                    let bgm_volume = window.get_bgm_volume() / 100.0;
                    bgm_player.play_loop(&format!("{}{}.ogg", BGM_PATH, bgm), volume * bgm_volume);
                }
            }
            Command::Choice(choices) => {
                let mut choose_branch = Vec::with_capacity(choices.len());
                for (index, choice) in choices.iter().enumerate() {
                    choose_branch.push((index as i32, SharedString::from(choice.0.clone())));
                }
                window.set_choose_branch(Rc::new(VecModel::from(choose_branch)).into());
                window.set_current_choose(choices.len() as i32);
            }
            Command::Dialogue { speaker, text } => {
                window.set_speaker(SharedString::from(speaker));
                window.set_dialogue(SharedString::from(text));
            }
            Command::PlayVoice(voice) => {
                let voice_player = voice_player.borrow_mut();
                let volume = window.get_main_volume() / 100.0;
                let voice_volume = window.get_voice_volume() / 100.0;
                voice_player.play_voice(
                    &format!("{}{}.ogg", VOICE_PATH, voice),
                    volume * voice_volume,
                );
            }
            Command::Figure {
                name,
                distance,
                body,
                face,
                position,
            } => {
                let body = Image::load_from_path(Path::new(&format!(
                    "{}{}/{}/{}.png",
                    FIGURE_PATH, name, distance, body
                )))
                .unwrap();
                let face = Image::load_from_path(Path::new(&format!(
                    "{}{}/{}/{}.png",
                    FIGURE_PATH, name, distance, face
                )))
                .unwrap();
                match &position[..] {
                    "0" => {
                        window.set_fg_body_0(body);
                        window.set_fg_face_0(face);
                    }
                    _ => (),
                }
            }
            Command::Jump(jump) => {
                let volume = window.get_main_volume() * window.get_bgm_volume() / 10000.0;
                execute_jump(script, bgm_player, volume, jump, weak_for_jump).await?;
            }
            Command::Label(label) => println!("{}", label),
        }
    };
    Ok(())
}

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use slint::Image;
use crate::audio::player::Player;
use crate::parser::script_parser::{Command, Commands};
use crate::script::Script;
use crate::ui::ui::MainWindow;
use slint::SharedString;
use crate::error::EngineError;

static BACKGROUND_PATH: &str = "./source/background/";
static VOICE_PATH: &str = "./source/voice/";
static BGM_PATH: &str = "./source/bgm/";
static FIGURE_PATH: &str = "./source/figure/";

pub async fn execute_script(script: Rc<RefCell<Script>>, bgm_player: Rc<RefCell<Player>>, voice_player: Rc<RefCell<Player>>, weak: slint::Weak<MainWindow>) -> Result<(), EngineError> {
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
            apply_command(command, script, bgm_player, voice_player, weak)?;
        }
        Commands::VarCommands(vars) => {
            for command in vars {
                let script = script.clone();
                let bgm_player = bgm_player.clone();
                let voice_player = voice_player.clone();
                let weak = weak.clone();
                apply_command(command, script, bgm_player, voice_player, weak)?;
            }
        }
    }
    Ok(())
}

fn apply_command(command: Command, script: Rc<RefCell<Script>>, bgm_player: Rc<RefCell<Player>>, voice_player: Rc<RefCell<Player>>, weak: slint::Weak<MainWindow>) -> Result<(), EngineError> {
    if let Some(window) = weak.upgrade() {
        match command {
            Command::SetBackground(bg) => {
                let image = Image::load_from_path(Path::new(&format!(
                    "{}{}.png",
                    BACKGROUND_PATH, bg
                )))
                    .unwrap();
                window.set_bg(image);
            },
            Command::PlayBgm(bgm) => {
                let mut bgm_player = bgm_player.borrow_mut();
                let volume = window.get_main_volume() / 100.0;
                let bgm_volume = window.get_bgm_volume() / 100.0;
                bgm_player.play_loop(&format!("{}{}.ogg", BGM_PATH, bgm), volume * bgm_volume);
                //println!("{:?}", time.elapsed());
            }
            Command::Dialogue { speaker, text } => {
                window.set_speaker(SharedString::from(speaker));
                window.set_dialogue(SharedString::from(text));
                //println!("{:?}", time.elapsed());
            }
            Command::PlayVoice(voice) => {
                let mut voice_player = voice_player.borrow_mut();
                let volume = window.get_main_volume() / 100.0;
                let voice_volume = window.get_voice_volume() / 100.0;
                voice_player.play_voice(&format!("{}{}.ogg", VOICE_PATH, voice), volume * voice_volume);
                //println!("{:?}", time.elapsed());
            }
            Command::Figure {name, distance,body, face, position} => {
                let body = Image::load_from_path(Path::new(&format!(
                    "{}{}/{}/{}.png",
                    FIGURE_PATH, name, distance, body
                ))).unwrap();
                let face = Image::load_from_path(Path::new(&format!(
                    "{}{}/{}/{}.png",
                    FIGURE_PATH, name, distance, face
                ))).unwrap();
                match &position[..] {
                    "0" => {
                        window.set_fg_body_0(body);
                        window.set_fg_face_0(face);
                    }
                    _ => ()
                }
            }
            Command::Jump(jump) => {
                let mut script = script.borrow_mut();
                if jump.0 == script.name() {
                    let mut current_block = script.index();
                    if let Some(index) = script.find_label(&jump.1) {
                        current_block = *index;
                    }
                    script.set_index(current_block);
                } else {
                    let mut jump_script = Script::from_name(jump.0)?;
                    let mut current_block = 0;
                    if let Some(index) = jump_script.find_label(&jump.1) {
                        current_block = *index;
                    }
                    jump_script.set_index(current_block);
                    *script = jump_script;
                }
            }
            Command::Label(label) => println!("{}",label),
            _ => {}
        }
    };
    Ok(())
}

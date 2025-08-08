use crate::parser::script_parser::{Command, Commands};
use crate::script::Script;
use crate::ui::ui::UiRenderBlock;

pub fn execute_script(script: &mut Script) -> Option<UiRenderBlock> {
    let mut block = UiRenderBlock::default();

    if let Some(commands) = script.next_command() {
        match commands {
            Commands::OneCommand(command) => {
                apply_command(&command, &mut block);
            }
            Commands::VarCommands(vars) => {
                for command in vars {
                    apply_command(&command, &mut block);
                }
            }
            Commands::EmptyCommands => return execute_script(script),
        }
        Some(block)
    } else {
        None
    }
}

fn apply_command(command: &Command, block: &mut UiRenderBlock) {
    match command {
        Command::SetBackground(bg) => block.background = Some(bg.clone()),
        Command::PlayBgm(bgm) => block.bgm = Some(bgm.clone()),
        Command::Dialogue { speaker, text } => {
            block.dialogue = Some((speaker.clone(), text.clone()))
        }
        Command::PlayVoice(v) => block.voice = Some(v.clone()),
        Command::Figure {name, distance,body, face, position} => block.figure = Some((name.to_string(), distance.to_string(), body.to_string(), face.to_string(), position.to_string())),
        _ => {}
    }
}

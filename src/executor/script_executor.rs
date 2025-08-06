use crate::parser::script_parser::{Command, Commands};
use crate::script::Script;

pub fn execute_script(script:&mut Script) {
    while let Some(commands) = script.next_command() {
        match commands {
            Commands::OneCommand(command) => {
                println!("Block");
                execute_commands(&command);
            }
            Commands::VarCommands(vars) => {
                println!("Block");
                for command in vars {
                    execute_commands(&command);
                }
            }
            Commands::EmptyCommands => ()
        }
    }
}

fn execute_commands(command:&Command) {
    match command {
        Command::SetBackground(bg) => println!("[背景] -> {}", bg),
        Command::PlayBgm(bgm) => println!("[播放BGM] -> {}", bgm),
        Command::Dialogue { speaker, text } => println!("{}：「{}」", speaker, text),
        Command::PlayVoice(v) => println!("[播放语音] -> {}", v),
        Command::Choice(choices) => {
            println!("[分支选项]");
            for (i, (label, jump)) in choices.iter().enumerate() {
                println!("  {}. {} -> {}", i + 1, label, jump);
            }
        }
        Command::Jump(to) => println!("[跳转] -> {}", to),
        _ => (),
    }
}

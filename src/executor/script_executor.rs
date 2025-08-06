use crate::parser::script_parser::Command;

pub fn execute_script(commands: &[Command]) {
    for cmd in commands {
        match cmd {
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
        }
    }
}
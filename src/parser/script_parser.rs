use crate::script::Script;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum Commands {
    OneCommand(Command),
    VarCommands(Vec<Command>),
    EmptyCommands,
}

#[derive(Debug, Clone)]
pub enum Command {
    SetBackground(String),
    PlayBgm(String),
    PlayVoice(String),
    Dialogue { speaker: String, text: String },
    Figure { name: String, body: String, face: String, position: String },
    Choice(Vec<(String, String)>),
    Jump(String),
    Label(String),
}

#[derive(Debug)]
pub enum ParserError {
    InvalidCommand { line: usize, content: String },
    MalformedDialogue { line: usize, content: String },
    UnknownLine { line: usize, content: String },
    EmptyBlock { line: usize },
    UnSupportedVersion { need: usize, indeed: String },
    TooShort { need: usize, indeed: usize },
}

static VERSION: usize = 1;

pub fn parse_script(text: &str) -> Result<Script, ParserError> {
    let mut commands = VecDeque::new();
    let mut block_lines = Vec::new();

    for (lineno, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            if !block_lines.is_empty() {
                let block = parse_block(&block_lines)?;
                commands.push_back(block);
                block_lines.clear();
            }
        } else {
            block_lines.push((lineno + 1, line.to_string()));
        }
    }

    if !block_lines.is_empty() {
        let block = parse_block(&block_lines)?;
        commands.push_back(block);
    }

    Ok(Script::from_commands(commands))
}

fn parse_block(lines: &[(usize, String)]) -> Result<Commands, ParserError> {
    use Command::*;
    use Commands::*;

    let mut commands = Vec::new();

    for (line_num, line) in lines {
        if line.starts_with('@') {
            if let Some((cmd, arg)) = line[1..].split_once(' ') {
                let cmd = match cmd {
                    "bg" => SetBackground(arg.to_string()),
                    "bgm" => PlayBgm(arg.to_string()),
                    "voice" => PlayVoice(arg.to_string()),
                    "fg" => {
                        let parts: Vec<&str> = arg.split('|').map(|s| s.trim()).collect();
                        if parts.len() != 4 {
                            return Err(ParserError::TooShort {need: 4, indeed: parts.len()});
                        }
                        Figure {name: parts[0].to_string(), body: parts[1].to_string(), face: parts[2].to_string(), position: parts[3].to_string()}
                    }
                    "jump" => Jump(arg.to_string()),
                    "label" => Label(arg.to_string()),
                    _ => {
                        return Err(ParserError::InvalidCommand {
                            line: *line_num,
                            content: line.clone(),
                        });
                    }
                };
                commands.push(cmd);
            } else {
                return Err(ParserError::InvalidCommand {
                    line: *line_num,
                    content: line.clone(),
                });
            }
        } else if line.starts_with('%') {
            if let Some((cmd, arg)) = line[1..].split_once(' ') {
                if cmd == "version" {
                    if !(arg.parse::<usize>().unwrap_or(0) == VERSION) {
                        return Err(ParserError::UnSupportedVersion {
                            need: VERSION,
                            indeed: arg.to_string(),
                        });
                    }
                    return Ok(EmptyCommands);
                } else {
                    return Err(ParserError::UnknownLine {
                        line: *line_num,
                        content: line.clone(),
                    });
                }
            } else {
                return Err(ParserError::InvalidCommand {
                    line: *line_num,
                    content: line.clone(),
                });
            }
        } else if line.starts_with('#') {
            continue;
        } else if let Some((speaker, text)) = line.split_once("“") {
            if let Some(text) = text.strip_suffix("”") {
                commands.push(Dialogue {
                    speaker: speaker.trim().to_string(),
                    text: text.trim().to_string(),
                });
                break;
            } else {
                return Err(ParserError::MalformedDialogue {
                    line: *line_num,
                    content: line.clone(),
                });
            }
        } else {
            return Err(ParserError::UnknownLine {
                line: *line_num,
                content: line.clone(),
            });
        }
    }

    if commands.is_empty() {
        return Err(ParserError::EmptyBlock { line: lines[0].0 });
    }

    Ok(if commands.len() == 1 {
        OneCommand(commands.into_iter().next().unwrap())
    } else {
        VarCommands(commands)
    })
}

use crate::error::EngineError;
use crate::script::Script;
use std::collections::{HashMap, HashSet};

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
    Dialogue {
        speaker: String,
        text: String,
    },
    Figure {
        name: String,
        distance: String,
        body: String,
        face: String,
        position: String,
    },
    Choice(Vec<(String, String)>),
    Jump((String, String)),
    Label(String),
}

#[derive(Debug)]
pub enum ParserError {
    InvalidCommand { line: usize, content: String },
    MalformedDialogue { line: usize, content: String },
    UnknownLine { line: usize, content: String },
    EmptyBlock { line: usize },
    UnSupportedVersion { need: usize, indeed: String },
    NoLabel,
    TooShort,
}

static VERSION: usize = 1;

type Commands_and_Labels = (Vec<Commands>, HashMap<String, usize>);
type Command_and_Label = (Commands, HashSet<String>);

pub fn parse_script(text: &str, script_name: &str) -> Result<Commands_and_Labels, EngineError> {
    let mut commands = Vec::new();
    let mut labels = HashMap::new();

    let mut block_lines = Vec::new();
    let mut block_index = 0;

    for (lineno, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            if !block_lines.is_empty() {
                let (block, label) = parse_block(&block_lines, script_name)?;
                for la in label {
                    labels.insert(la.to_string(), block_index);
                }
                block_index += 1;
                commands.push(block);
                block_lines.clear();
            }
        } else {
            block_lines.push((lineno + 1, line.to_string()));
        }
    }

    if !block_lines.is_empty() {
        let (block, label) = parse_block(&block_lines, script_name)?;
        for la in label {
            labels.insert(la.to_string(), block_index);
        }
        commands.push(block);
    }

    Ok((commands, labels))
}

fn parse_block(
    lines: &[(usize, String)],
    script_name: &str,
) -> Result<Command_and_Label, EngineError> {
    use Command::*;
    use Commands::*;

    let mut commands = Vec::new();
    let mut label = HashSet::new();

    for (line_num, line) in lines {
        if line.starts_with('@') {
            if let Some((cmd, arg)) = line[1..].split_once(' ') {
                let cmd = match cmd {
                    "bg" => SetBackground(arg.to_string()),
                    "bgm" => PlayBgm(arg.to_string()),
                    "voice" => PlayVoice(arg.to_string()),
                    "fg" => {
                        let mut parts = arg.split('|').map(str::trim);
                        match (
                            parts.next(),
                            parts.next(),
                            parts.next(),
                            parts.next(),
                            parts.next(),
                        ) {
                            (
                                Some(name),
                                Some(distance),
                                Some(body),
                                Some(face),
                                Some(position),
                            ) => Figure {
                                name: name.to_string(),
                                distance: distance.to_string(),
                                body: body.to_string(),
                                face: face.to_string(),
                                position: position.to_string(),
                            },
                            _ => return Err(EngineError::from(ParserError::TooShort)),
                        }
                    }
                    "jump" => match arg.split_once(":") {
                        Some((name, label)) if !name.is_empty() && !label.is_empty() => {
                            Jump((name.to_string(), label.to_string()))
                        }
                        Some((name, "")) if !name.is_empty() => {
                            Jump((name.to_string(), "start".to_string()))
                        }
                        Some(("", label)) => Jump((script_name.to_string(), label.to_string())),
                        None => Jump((arg.to_string(), "start".to_string())),
                        _ => unreachable!(),
                    },
                    "label" => {
                        label.insert(arg.to_string());
                        Label(arg.to_string())
                    }
                    _ => {
                        return Err(EngineError::from(ParserError::InvalidCommand {
                            line: *line_num,
                            content: line.clone(),
                        }));
                    }
                };
                commands.push(cmd);
            } else {
                return Err(EngineError::from(ParserError::InvalidCommand {
                    line: *line_num,
                    content: line.clone(),
                }));
            }
        } else if line.starts_with('%') {
            if let Some((cmd, arg)) = line[1..].split_once(' ') {
                if cmd == "version" {
                    if !(arg.parse::<usize>().unwrap_or(0) == VERSION) {
                        return Err(EngineError::from(ParserError::UnSupportedVersion {
                            need: VERSION,
                            indeed: arg.to_string(),
                        }));
                    }
                    return Ok((EmptyCommands, label));
                } else {
                    return Err(EngineError::from(ParserError::UnknownLine {
                        line: *line_num,
                        content: line.clone(),
                    }));
                }
            } else {
                return Err(EngineError::from(ParserError::InvalidCommand {
                    line: *line_num,
                    content: line.clone(),
                }));
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
                return Err(EngineError::from(ParserError::MalformedDialogue {
                    line: *line_num,
                    content: line.clone(),
                }));
            }
        } else {
            return Err(EngineError::from(ParserError::UnknownLine {
                line: *line_num,
                content: line.clone(),
            }));
        }
    }

    if commands.is_empty() {
        return Err(EngineError::from(ParserError::EmptyBlock {
            line: lines[0].0,
        }));
    }

    Ok(if commands.len() == 1 {
        (OneCommand(commands.into_iter().next().unwrap()), label)
    } else {
        (VarCommands(commands), label)
    })
}

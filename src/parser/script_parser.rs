use crate::error::EngineError;
use crate::script::{Label, Script};
use std::collections::{BTreeMap, HashMap, HashSet};

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
    Choice(HashMap<String, Label>),
    Jump(Label),
    Label(String),
}

#[derive(Debug)]
pub enum ParserError {
    ChooseError(String),
    InvalidCommand { line: usize, content: String },
    MalformedDialogue { line: usize, content: String },
    UnknownLine { line: usize, content: String },
    EmptyBlock { line: usize },
    UnSupportedVersion { need: usize, indeed: String },
    TooShort,
}

static VERSION: usize = 1;

pub fn parse_script(
    text: &str,
    script_name: &str,
    commands: &mut Vec<Commands>,
    labels: &mut HashMap<String, usize>,
    choices: &mut HashMap<String, Label>,
    bgms: &mut BTreeMap<usize, String>,
) -> Result<(), EngineError> {
    let mut block_lines = Vec::new();
    let mut block_index = 0;

    for (lineno, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            if !block_lines.is_empty() {
                parse_block(
                    &block_lines,
                    script_name,
                    &mut block_index,
                    commands,
                    labels,
                    choices,
                    bgms,
                )?;
                block_lines.clear();
            }
        } else {
            block_lines.push((lineno + 1, line.to_string()));
        }
    }

    if !block_lines.is_empty() {
        parse_block(
            &block_lines,
            script_name,
            &mut block_index,
            commands,
            labels,
            choices,
            bgms,
        )?;
    }

    Ok(())
}

fn parse_block(
    lines: &[(usize, String)],
    script_name: &str,
    block_index: &mut usize,
    commands: &mut Vec<Commands>,
    labels: &mut HashMap<String, usize>,
    choices: &mut HashMap<String, Label>,
    bgms: &mut BTreeMap<usize, String>,
) -> Result<(), EngineError> {
    use Command::*;
    use Commands::*;

    let mut block_commands = Vec::new();

    for (line_num, line) in lines {
        if line.starts_with('@') {
            if let Some((cmd, arg)) = line[1..].split_once(' ') {
                let cmd = match cmd {
                    "bg" => SetBackground(arg.to_string()),
                    "bgm" => {
                        bgms.insert(*block_index, arg.to_string());
                        PlayBgm(arg.to_string())
                    }
                    "choose" => {
                        let num = arg.parse::<usize>()?;
                        let mut choose_branch = HashMap::with_capacity(num);
                        for i in 1..=num {
                            if let Some((choice, script)) = lines[i].1.split_once(' ') {
                                let (choice, label) = match script.split_once(":") {
                                    Some((name, label))
                                        if !name.is_empty() && !label.is_empty() =>
                                    {
                                        (choice.to_string(), (name.to_string(), label.to_string()))
                                    }
                                    Some((name, "")) if !name.is_empty() => (
                                        choice.to_string(),
                                        (name.to_string(), "start".to_string()),
                                    ),
                                    Some(("", label)) => (
                                        choice.to_string(),
                                        (script_name.to_string(), label.to_string()),
                                    ),
                                    None => (
                                        choice.to_string(),
                                        (script.to_string(), "start".to_string()),
                                    ),
                                    _ => unreachable!(),
                                };
                                choose_branch.insert(choice.clone(), label.clone());
                                choices.insert(choice, label);
                            } else {
                                return Err(EngineError::from(ParserError::ChooseError(format!(
                                    "Invalid choice: {}",
                                    lines[line_num + i].1
                                ))));
                            }
                        }
                        block_commands.push(Choice(choose_branch));
                        break;
                    }
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
                        labels.insert(arg.to_string(), *block_index);
                        Label(arg.to_string())
                    }
                    _ => {
                        return Err(EngineError::from(ParserError::InvalidCommand {
                            line: *line_num,
                            content: line.clone(),
                        }));
                    }
                };
                block_commands.push(cmd);
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
                block_commands.push(Dialogue {
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

    // if block_commands.is_empty() {
    //     return Err(EngineError::from(ParserError::EmptyBlock {
    //         line: lines[0].0,
    //     }));
    // }

    if block_commands.len() == 1 {
        *block_index += 1;
        commands.push(OneCommand(block_commands.into_iter().next().unwrap()));
    } else if block_commands.len() > 1 {
        *block_index += 1;
        commands.push(VarCommands(block_commands))
    }

    Ok(())
}

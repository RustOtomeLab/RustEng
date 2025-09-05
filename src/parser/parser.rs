use crate::error::EngineError;
use crate::script::{Label, Script};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Commands {
    OneCmd(Command),
    VarCmds(Vec<Command>),
    EmptyCmd,
}

#[derive(Debug, Clone)]
pub enum Command {
    SetBackground(String),
    PlayBgm(String),
    PlayVoice {
        name: String,
        voice: String,
    },
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
        delay: Option<String>,
    },
    Clear(String, String),
    Choice((String, HashMap<String, Label>)),
    Jump(Label),
    Label(String),
    Empty,
}

impl Command {
    fn latest_fg(&self, index: &usize, dis: &str, pos: &str, script: &Script) -> Option<Command> {
        if let Command::Figure {
            name,
            distance,
            body,
            face,
            position,
            delay,
            ..
        } = self
        {
            return match (!body.is_empty(), delay.is_some()) {
                (false, true) => {
                    let (body, face) = script.find_latest_fg(index, dis, pos);
                    Some(Command::Figure {
                        name: name.clone(),
                        distance: distance.clone(),
                        body,
                        face,
                        position: position.clone(),
                        delay: None,
                    })
                }
                (false, false) => {
                    let (body, _) = script.find_latest_fg(index, dis, pos);
                    Some(Command::Figure {
                        name: name.clone(),
                        distance: distance.clone(),
                        body,
                        face: face.clone(),
                        position: position.clone(),
                        delay: None,
                    })
                }
                (true, true) => None,
                (true, false) => Some(self.clone()),
            };
        }
        unreachable!()
    }
}

#[derive(Debug)]
pub enum ParserError {
    ChooseError(String),
    InvalidCommand { line: usize, content: String },
    MalformedDialogue { line: usize, content: String },
    UnknownLine { line: usize, content: String },
    UnSupportedVersion { need: usize, indeed: String },
    TooShort,
}

static VERSION: usize = 1;

impl Script {
    pub fn parse_script(&mut self, text: &str) -> Result<(), EngineError> {
        let mut block_lines = Vec::new();
        let mut block_index = 0;

        for (lineno, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                if !block_lines.is_empty() {
                    self.parse_block(&block_lines, &mut block_index)?;
                    block_lines.clear();
                }
            } else {
                block_lines.push((lineno + 1, line.to_string()));
            }
        }

        if !block_lines.is_empty() {
            self.parse_block(&block_lines, &mut block_index)?;
        }

        Ok(())
    }

    fn parse_block(
        &mut self,
        lines: &[(usize, String)],
        block_index: &mut usize,
    ) -> Result<(), EngineError> {
        use Command::*;
        use Commands::*;

        let mut block_commands = Vec::new();

        for (index, (line_num, line)) in lines.into_iter().enumerate() {
            if let Some(line) = line.strip_prefix('@') {
                if let Some((cmd, arg)) = line.split_once(' ') {
                    let cmd = match cmd {
                        "bg" => {
                            self.backgrounds.insert(*block_index, arg.to_string());
                            SetBackground(arg.to_string())
                        }
                        "bgm" => {
                            self.bgms.insert(*block_index, arg.to_string());
                            PlayBgm(arg.to_string())
                        }
                        "choose" => {
                            let num = arg.parse::<usize>()?;
                            let mut choose_branch = HashMap::with_capacity(num);
                            let explain = lines[index + 1].1.clone();
                            for i in index + 2..=index + num + 1 {
                                if let Some((choice, script)) = lines[i].1.split_once(' ') {
                                    let (choice, label) = match script.split_once(":") {
                                        Some((name, label))
                                            if !name.is_empty() && !label.is_empty() =>
                                        {
                                            (
                                                choice.to_string(),
                                                (name.to_string(), label.to_string()),
                                            )
                                        }
                                        Some((name, "")) if !name.is_empty() => (
                                            choice.to_string(),
                                            (name.to_string(), "start".to_string()),
                                        ),
                                        Some(("", label)) => (
                                            choice.to_string(),
                                            (self.name.to_string(), label.to_string()),
                                        ),
                                        None => (
                                            choice.to_string(),
                                            (script.to_string(), "start".to_string()),
                                        ),
                                        _ => unreachable!(),
                                    };
                                    choose_branch.insert(choice.clone(), label.clone());
                                    self.choices.insert(choice, label);
                                } else {
                                    return Err(EngineError::from(ParserError::ChooseError(
                                        format!("Invalid choice: {}", lines[line_num + i].1),
                                    )));
                                }
                            }
                            block_commands.push(Choice((explain, choose_branch)));
                            break;
                        }
                        "voice" => {
                            if let Some((name, voice)) = arg.split_once('|') {
                                PlayVoice {
                                    name: name.to_string(),
                                    voice: voice.to_string(),
                                }
                            } else {
                                return Err(EngineError::from(ParserError::TooShort));
                            }
                        }
                        "fg" => {
                            let mut parts = arg.split('|').map(str::trim);
                            match (
                                parts.next(),
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
                                    delay,
                                ) => {
                                    let command = Figure {
                                        name: name.to_string(),
                                        distance: distance.to_string(),
                                        body: body.to_string(),
                                        face: face.to_string(),
                                        position: position.to_string(),
                                        delay: delay.map(|d| d.to_string()),
                                    };
                                    if let Some(cmd) =
                                        command.latest_fg(block_index, distance, position, self)
                                    {
                                        self.figures
                                            .entry(*block_index)
                                            .or_insert_with(Vec::new)
                                            .push(cmd);
                                    }
                                    command
                                }
                                _ => return Err(EngineError::from(ParserError::TooShort)),
                            }
                        }
                        "clear" => {
                            if let Some((dis, pos)) = arg.split_once("|") {
                                Clear(dis.to_string(), pos.to_string())
                            } else {
                                if arg == "All" {
                                    Clear(arg.to_string(), arg.to_string())
                                } else {
                                    return Err(EngineError::from(ParserError::InvalidCommand {
                                        line: *line_num,
                                        content: line.to_string(),
                                    }));
                                }
                            }
                        }
                        "jump" => match arg.split_once(":") {
                            Some((name, label)) if !name.is_empty() && !label.is_empty() => {
                                Jump((name.to_string(), label.to_string()))
                            }
                            Some((name, "")) if !name.is_empty() => {
                                Jump((name.to_string(), "start".to_string()))
                            }
                            Some(("", label)) => Jump((self.name.to_string(), label.to_string())),
                            None => Jump((arg.to_string(), "start".to_string())),
                            _ => unreachable!(),
                        },
                        "label" => {
                            self.labels.insert(arg.to_string(), *block_index);
                            Label(arg.to_string())
                        }
                        _ => {
                            return Err(EngineError::from(ParserError::InvalidCommand {
                                line: *line_num,
                                content: line.to_string(),
                            }));
                        }
                    };
                    block_commands.push(cmd);
                } else {
                    return Err(EngineError::from(ParserError::InvalidCommand {
                        line: *line_num,
                        content: line.to_string(),
                    }));
                }
            } else if let Some(line) = line.strip_prefix('%') {
                if let Some((cmd, arg)) = line.split_once(' ') {
                    if cmd == "version" {
                        if arg.parse::<usize>().unwrap_or(0) != VERSION {
                            return Err(EngineError::from(ParserError::UnSupportedVersion {
                                need: VERSION,
                                indeed: arg.to_string(),
                            }));
                        }
                    } else {
                        return Err(EngineError::from(ParserError::UnknownLine {
                            line: *line_num,
                            content: line.to_string(),
                        }));
                    }
                } else {
                    return Err(EngineError::from(ParserError::InvalidCommand {
                        line: *line_num,
                        content: line.to_string(),
                    }));
                }
            } else if let Some(_) = line.strip_prefix('#') {
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

        if block_commands.len() == 1 {
            *block_index += 1;
            self.commands
                .push(OneCmd(block_commands.into_iter().next().unwrap()));
        } else if block_commands.len() > 1 {
            *block_index += 1;
            self.commands.push(VarCmds(block_commands))
        }

        Ok(())
    }
}

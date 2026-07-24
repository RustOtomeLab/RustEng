use crate::config::ENGINE_CONFIG;
use crate::error::{EngineError, ScriptError};
use crate::script::{Label, Script};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
pub(crate) enum Commands {
    OneCmd(Command),
    VarCmds(Vec<Command>),
    EmptyCmd,
}

#[derive(Debug, Clone)]
pub(crate) enum Command {
    Background {
        name: String,
        x_offset: Option<f32>,
        y_offset: Option<f32>,
        zoom: Option<f32>,
        is_cg: bool,
    },
    PlayBgm(String),
    PlayVoice {
        name: String,
        voice: String,
    },
    PlayVideo(String),
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
    Move {
        name: String,
        distance: String,
        position: String,
        action: String,
        repeat: i32,
        delay: Option<String>,
    },
    Clear(String),
    Choice((String, HashMap<String, Label>)),
    Jump(Label),
    Label,
}

impl Command {
    pub(crate) fn delete_delay(&mut self) {
        if let Command::Figure { delay, .. } | Command::Move { delay, .. } = self {
            delay.take();
        }
    }

    pub(crate) fn change_position(&mut self, pos: &str) {
        if let Command::Figure { position, .. } | Command::Move { position, .. } = self {
            *position = pos.to_string();
        }
    }

    pub(crate) fn action(&self) -> &String {
        if let Command::Move { action, .. } = self {
            action
        } else {
            unreachable!()
        }
    }

    pub(crate) fn back(&self) -> Command {
        if let Command::Move {
            name,
            distance,
            position,
            ..
        } = self
        {
            Command::Move {
                name: name.to_string(),
                distance: distance.to_string(),
                position: position.to_string(),
                action: "back".to_string(),
                repeat: 1,
                delay: Some("150".to_string()),
            }
        } else {
            unreachable!()
        }
    }
}

static VERSION: usize = 1;

pub(crate) struct Parser {
    script: Script,
    block_index: usize,
}

impl Parser {
    pub(crate) fn new(name: &str) -> Parser {
        let mut script = Script::new();
        script.set_name(name);
        Parser {
            script,
            block_index: 0,
        }
    }

    pub(crate) fn load(name: &str) -> Result<Script, EngineError> {
        let path = format!("{}{}.reg", ENGINE_CONFIG.script_path(), name);
        let text = fs::read_to_string(&path).map_err(|e| ScriptError::ReadFile {
            path: path.clone(),
            source: e,
        })?;
        Parser::new(name).parse(&text)
    }

    pub(crate) fn parse(mut self, text: &str) -> Result<Script, EngineError> {
        let mut block_lines = Vec::new();

        for (lineno, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                if !block_lines.is_empty() {
                    self.parse_block(&block_lines)?;
                    block_lines.clear();
                }
            } else {
                block_lines.push((lineno + 1, line.to_string()));
            }
        }

        if !block_lines.is_empty() {
            self.parse_block(&block_lines)?;
        }

        Ok(self.script)
    }

    fn parse_block(&mut self, lines: &[(usize, String)]) -> Result<(), EngineError> {
        use Command::*;
        use Commands::*;

        let mut block_commands = Vec::new();

        for (index, (line_num, line)) in lines.iter().enumerate() {
            if let Some(line) = line.strip_prefix('@') {
                if let Some((cmd, arg)) = line.split_once(' ') {
                    let cmd = match cmd {
                        "bg" | "cg" => {
                            let mut parts = arg.split('|').map(str::trim);
                            let bg = Background {
                                name: parts.next().unwrap_or("").to_string(),
                                x_offset: parts.next().and_then(|s| s.parse::<f32>().ok()),
                                y_offset: parts.next().and_then(|s| s.parse::<f32>().ok()),
                                zoom: parts.next().and_then(|s| s.parse::<f32>().ok()),
                                is_cg: cmd == "cg",
                            };
                            self.script.insert_background(self.block_index, bg.clone());
                            bg
                        }
                        "bgm" => {
                            self.script.insert_bgm(self.block_index, arg.to_string());
                            PlayBgm(arg.to_string())
                        }
                        "choose" => {
                            let num = arg.parse::<usize>().map_err(ScriptError::from)?;
                            let mut choose_branch = HashMap::with_capacity(num);
                            let explain = lines[index + 1].1.clone();
                            for (i, line) in lines.iter().take(index + num + 1 + 1).skip(index + 2)
                            {
                                if let Some((choice, script)) = line.split_once(' ') {
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
                                            (self.script.name().to_string(), label.to_string()),
                                        ),
                                        None => (
                                            choice.to_string(),
                                            (script.to_string(), "start".to_string()),
                                        ),
                                        _ => unreachable!(),
                                    };
                                    choose_branch.insert(choice.clone(), label.clone());
                                    self.script.insert_choice(choice, label);
                                } else {
                                    return Err(EngineError::from(ScriptError::Choice(format!(
                                        "Invalid choice at line {i}: {line}"
                                    ))));
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
                                return Err(EngineError::from(ScriptError::ArgsTooShort {
                                    cmd: "voice".to_string(),
                                    line: *line_num,
                                    content: line.to_string(),
                                }));
                            }
                        }
                        "video" => {
                            let name = arg.trim();
                            if name.is_empty() {
                                return Err(EngineError::from(ScriptError::ArgsTooShort {
                                    cmd: "video".to_string(),
                                    line: *line_num,
                                    content: line.to_string(),
                                }));
                            }
                            PlayVideo(name.to_string())
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
                                        delay: None,
                                    };
                                    self.script.update_figures(
                                        self.block_index,
                                        distance,
                                        position,
                                        command,
                                    );
                                    Figure {
                                        name: name.to_string(),
                                        distance: distance.to_string(),
                                        body: body.to_string(),
                                        face: face.to_string(),
                                        position: position.to_string(),
                                        delay: delay.map(|d| d.to_string()),
                                    }
                                }
                                _ => {
                                    return Err(EngineError::from(ScriptError::ArgsTooShort {
                                        cmd: "fg".to_string(),
                                        line: *line_num,
                                        content: line.to_string(),
                                    }))
                                }
                            }
                        }
                        "move" => {
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
                                    Some(position),
                                    Some(action),
                                    Some(repeat),
                                    delay,
                                ) => {
                                    let command = Move {
                                        name: name.to_string(),
                                        distance: distance.to_string(),
                                        position: position.to_string(),
                                        action: action.to_string(),
                                        repeat: repeat.parse::<i32>().map_err(ScriptError::from)?,
                                        delay: delay.map(|d| d.to_string()),
                                    };
                                    if action.contains("to") {
                                        let mut cmd = self.script.change_figure(
                                            self.block_index,
                                            distance,
                                            position,
                                        );
                                        cmd.change_position(position);
                                        let (_, pos) = action.split_once('o').unwrap();
                                        self.script.update_figures(
                                            self.block_index,
                                            distance,
                                            pos,
                                            cmd,
                                        );
                                    }
                                    command
                                }
                                _ => {
                                    return Err(EngineError::from(ScriptError::ArgsTooShort {
                                        cmd: "move".to_string(),
                                        line: *line_num,
                                        content: line.to_string(),
                                    }))
                                }
                            }
                        }
                        "clear" => {
                            self.script.insert_clear(self.block_index);
                            Clear(arg.to_string())
                        }
                        "jump" => match arg.split_once(":") {
                            Some((name, label)) if !name.is_empty() && !label.is_empty() => {
                                Jump((name.to_string(), label.to_string()))
                            }
                            Some((name, "")) if !name.is_empty() => {
                                Jump((name.to_string(), "start".to_string()))
                            }
                            Some(("", label)) => {
                                Jump((self.script.name().to_string(), label.to_string()))
                            }
                            None => Jump((arg.to_string(), "start".to_string())),
                            _ => unreachable!(),
                        },
                        "label" => {
                            self.script.insert_label(arg.to_string(), self.block_index);
                            Label
                        }
                        _ => {
                            return Err(EngineError::from(ScriptError::InvalidCommand {
                                line: *line_num,
                                content: line.to_string(),
                            }));
                        }
                    };
                    block_commands.push(cmd);
                } else {
                    return Err(EngineError::from(ScriptError::InvalidCommand {
                        line: *line_num,
                        content: line.to_string(),
                    }));
                }
            } else if let Some(line) = line.strip_prefix('%') {
                if let Some((cmd, arg)) = line.split_once(' ') {
                    if cmd == "version" {
                        if arg.parse::<usize>().unwrap_or(0) != VERSION {
                            return Err(EngineError::from(ScriptError::UnsupportedVersion {
                                need: VERSION,
                                indeed: arg.to_string(),
                            }));
                        }
                    } else {
                        return Err(EngineError::from(ScriptError::UnknownLine {
                            line: *line_num,
                            content: line.to_string(),
                        }));
                    }
                } else {
                    return Err(EngineError::from(ScriptError::InvalidCommand {
                        line: *line_num,
                        content: line.to_string(),
                    }));
                }
            } else if line.strip_prefix('#').is_some() {
                continue;
            } else if let Some((speaker, text)) = line.split_once("“") {
                if let Some(text) = text.strip_suffix("”") {
                    block_commands.push(Dialogue {
                        speaker: speaker.trim().to_string(),
                        text: text.trim().to_string(),
                    });
                    break;
                } else {
                    return Err(EngineError::from(ScriptError::MalformedDialogue {
                        line: *line_num,
                        content: line.clone(),
                    }));
                }
            } else {
                return Err(EngineError::from(ScriptError::UnknownLine {
                    line: *line_num,
                    content: line.clone(),
                }));
            }
        }

        if block_commands.len() == 1 {
            self.block_index += 1;
            self.script
                .push_command(OneCmd(block_commands.into_iter().next().unwrap()));
        } else if block_commands.len() > 1 {
            self.block_index += 1;
            self.script.push_command(VarCmds(block_commands))
        }

        Ok(())
    }
}

use crate::error::EngineError;
use crate::parser::parser::{parse_script, Commands};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use crate::audio::player::PreBgm;

pub type Label = (String, String);

struct Args {
    path: String,
}

impl Args {
    fn new(args: &str) -> Args {
        Args {
            path: format!("./source/script/{}.reg", args),
        }
    }
}

impl Default for Args {
    fn default() -> Self {
        Args {
            path: "./source/script/ky01.reg".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Script {
    name: String,
    explain: String,
    commands: Vec<Commands>,
    current_block: usize,
    bgms: BTreeMap<usize, String>,
    current_bgm: String,
    pre_bgm: PreBgm,
    backgrounds: BTreeMap<usize, String>,
    pre_bg: Option<String>,
    choices: HashMap<String, Label>,
    labels: HashMap<String, usize>,
}

impl Script {
    pub fn new() -> Script {
        Script {
            name: String::new(),
            explain: String::new(),
            commands: Vec::new(),
            current_block: 0,
            bgms: BTreeMap::new(),
            current_bgm: String::new(),
            pre_bgm: PreBgm::None,
            backgrounds: BTreeMap::new(),
            pre_bg: None,
            choices: HashMap::new(),
            labels: HashMap::new(),
        }
    }

    pub fn with_name(&mut self, name: &str) -> Result<(), EngineError> {
        self.name = name.to_string();
        let path = Args::new(&name);
        let script = fs::read_to_string(&path.path)?;
        parse_script(
            &script,
            &self.name,
            &mut self.commands,
            &mut self.labels,
            &mut self.choices,
            &mut self.bgms,
            &mut self.backgrounds,
        )
    }

    pub fn next_command(&mut self) -> Option<&Commands> {
        let command = self.commands.get(self.current_block);
        self.current_block += 1;
        command
    }

    pub fn set_explain(&mut self, explain: &String) {
        let mut explain = &explain[..];
        if explain.len() > 18 {
            explain = &explain[0..18];
        }
        self.explain = format!("{}{}", explain, "...");
    }

    pub fn set_index(&mut self, index: usize) {
        self.current_block = index;
    }

    pub fn set_current_bgm(&mut self, bgm: String) {
        self.current_bgm = bgm;
    }

    pub fn set_pre_bgm(&mut self, pre_bgm: PreBgm) {
        self.pre_bgm = pre_bgm;
    }

    pub fn set_pre_bg(&mut self, pre_bg: Option<String>) {
        if let Some(pre_bg) = pre_bg {
            self.pre_bg = Some(pre_bg);
        } else {
            self.pre_bg = None;
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn index(&self) -> usize {
        self.current_block
    }

    pub fn explain(&self) -> &str {
        &self.explain
    }

    pub fn current_bgm(&self) -> &str {
        &self.current_bgm
    }

    pub fn pre_bg(&mut self) -> Option<String> {
        self.pre_bg.take()
    }

    pub fn pre_bgm(&mut self) -> PreBgm {
        self.pre_bgm.clone()
    }

    pub fn find_label(&self, name: &str) -> Option<&usize> {
        self.labels.get(name)
    }

    pub fn get_choice_label(&self, name: &str) -> Option<&Label> {
        self.choices.get(name)
    }

    pub fn get_bgm(&self, index: usize) -> Option<(&usize, &String)> {
        self.bgms.range(..=index).next_back()
    }

    pub fn get_background(&self, index: usize) -> Option<(&usize, &String)> {
        self.backgrounds.range(..=index).next_back()
    }
}

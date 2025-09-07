use crate::audio::player::PreBgm;
use crate::config::ENGINE_CONFIG;
use crate::error::EngineError;
use crate::parser::parser::{Command, Commands};
use slint::{SharedString, ToSharedString};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;

pub type Label = (String, String);

const WINDOW_SIZE: usize = 4;

#[derive(Debug, Clone)]
pub struct BackLog {
    pub front: SharedString,
    pub back: SharedString,
    pub script: SharedString,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct Script {
    pub(crate) name: String,
    explain: String,
    backlog_offset: usize,
    backlog: Vec<BackLog>,
    pub(crate) commands: Vec<Commands>,
    current_block: usize,
    pub(crate) bgms: BTreeMap<usize, String>,
    current_bgm: String,
    pre_bgm: PreBgm,
    pub(crate) backgrounds: BTreeMap<usize, String>,
    pre_bg: Option<String>,
    figures: BTreeMap<usize, Figure>,
    pub(crate) clear: HashSet<usize>,
    pre_figures: Option<Figure>,
    pub(crate) choices: HashMap<String, Label>,
    pub(crate) labels: HashMap<String, usize>,
}

impl Script {
    pub fn new() -> Script {
        Script {
            name: String::new(),
            explain: String::new(),
            backlog_offset: 0,
            backlog: Vec::new(),
            commands: Vec::new(),
            current_block: 0,
            bgms: BTreeMap::new(),
            current_bgm: String::new(),
            pre_bgm: PreBgm::None,
            backgrounds: BTreeMap::new(),
            pre_bg: None,
            figures: BTreeMap::new(),
            clear: HashSet::new(),
            pre_figures: None,
            choices: HashMap::new(),
            labels: HashMap::new(),
        }
    }

    pub fn with_name(&mut self, name: &str) -> Result<(), EngineError> {
        self.name = name.to_string();
        let path = format!("{}{}.reg", ENGINE_CONFIG.script_path(), name);
        let script = fs::read_to_string(&path)?;
        self.parse_script(&script)?;
        //println!("{:#?}", self.commands);
        Ok(())
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

    pub fn set_offset(&mut self, offset: i32) {
        let new_offset = (self.backlog_offset as i32 + offset).max(0);
        // 不能超过最大可偏移量
        let max_offset = self.max_offset();
        self.backlog_offset = new_offset.min(max_offset as i32) as usize;
    }

    fn max_offset(&self) -> usize {
        self.backlog.len().saturating_sub(WINDOW_SIZE)
    }

    pub fn set_current_bgm(&mut self, bgm: String) {
        self.current_bgm = bgm;
    }

    pub fn set_pre_bgm(&mut self, pre_bgm: PreBgm) {
        self.pre_bgm = pre_bgm;
    }

    pub fn set_pre_bg(&mut self, pre_bg: Option<String>) {
        self.pre_bg = pre_bg;
    }

    pub fn set_pre_figures(&mut self, pre_figures: Option<Figure>) {
        self.pre_figures = pre_figures;
    }

    pub fn update_figures(
        &mut self,
        index: usize,
        distance: &str,
        position: &str,
        command: Command,
    ) {
        self.figures
            .entry(index)
            .or_insert_with(Figure::default)
            .push(distance, position, command);
    }

    pub fn set_backlog(&mut self, backlog: Vec<BackLog>) {
        self.backlog = backlog;
    }

    pub fn push_backlog(&mut self, name: SharedString, text: SharedString) {
        self.backlog.push(BackLog {
            front: name,
            back: text,
            script: self.name.to_shared_string(),
            index: self.current_block,
        });
    }

    pub fn backlog(&self) -> Vec<(SharedString, SharedString, i32, SharedString)> {
        let total = self.backlog.len();
        if total == 0 {
            return vec![];
        }

        let end = total.saturating_sub(self.backlog_offset);
        let start = end.saturating_sub(WINDOW_SIZE);
        self.backlog[start..end]
            .iter()
            .map(|backlog| {
                (
                    backlog.back.to_shared_string(),
                    backlog.front.to_shared_string(),
                    backlog.index as i32,
                    backlog.script.to_shared_string(),
                )
            })
            .collect()
    }

    pub fn take_backlog(self) -> Vec<BackLog> {
        self.backlog
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
        let bgm = self.pre_bgm.clone();
        self.pre_bgm = PreBgm::None;
        bgm
    }

    pub fn pre_figures(&mut self) -> Option<Figure> {
        self.pre_figures.take()
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

    pub fn get_figures(&self, index: usize) -> Option<(&usize, &Figure)> {
        self.figures.range(..=index).next_back()
    }

    pub fn change_figure(&mut self, index: usize, distance: &str, position: &str) -> Command {
        let pos = format!("{}{}", distance, position);
        let mut idx = 0;
        for i in (0..=index).rev() {
            if let Some(fg) = self.figures.get(&i) {
                if fg.0.get(&pos).is_some() {
                    idx = i;
                    break;
                }
            }
        }
        let figure = self.figures.get_mut(&idx).unwrap();
        figure.0.remove(&pos).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct Figure(pub HashMap<String, Command>);

impl Default for Figure {
    fn default() -> Self {
        Figure(HashMap::new())
    }
}

impl Figure {
    fn push(&mut self, distance: &str, position: &str, command: Command) {
        self.0.insert(format!("{}{}", distance, position), command);
    }
}

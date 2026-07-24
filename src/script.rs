use crate::media::player::PreBgm;
use crate::media::player::PreBgm::Play;
use crate::parser::script_parser::{Command, Commands};
use crate::ui::initialize::BackLogItem;
use slint::{SharedString, ToSharedString};
use std::collections::{BTreeMap, HashMap, HashSet};

pub(crate) type Label = (String, String);

#[derive(Debug, Clone, Default)]
pub(crate) struct PreItems {
    pre_bg: Option<Command>,
    pre_bgm: PreBgm,
    pre_figures: Option<Figure>,
}

const WINDOW_SIZE: usize = 4;

#[derive(Debug, Clone)]
pub(crate) struct Script {
    name: String,
    explain: String,
    backlog_offset: usize,
    backlog: Vec<BackLogItem>,
    commands: Vec<Commands>,
    current_block: usize,
    bgm: BTreeMap<usize, String>,
    current_bgm: String,
    pre_voice: Option<(SharedString, SharedString)>,
    backgrounds: BTreeMap<usize, Command>,
    figures: BTreeMap<usize, Figure>,
    clear: HashSet<usize>,
    choices: HashMap<String, Label>,
    labels: HashMap<String, usize>,
    pre_items: PreItems,
}

impl Script {
    pub(crate) fn new() -> Script {
        Script {
            name: String::new(),
            explain: String::new(),
            backlog_offset: 0,
            backlog: Vec::new(),
            commands: Vec::new(),
            current_block: 0,
            bgm: BTreeMap::new(),
            current_bgm: String::new(),
            pre_voice: None,
            backgrounds: BTreeMap::new(),
            figures: BTreeMap::new(),
            clear: HashSet::new(),
            choices: HashMap::new(),
            labels: HashMap::new(),
            pre_items: PreItems::default(),
        }
    }

    pub(crate) fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub(crate) fn next_command(&mut self) -> Option<&Commands> {
        let command = self.commands.get(self.current_block);
        self.current_block += 1;
        command
    }

    pub(crate) fn set_explain(&mut self, explain: &str) {
        let mut explain = explain;
        if explain.len() > 18 {
            explain = &explain[0..18];
        }
        self.explain = format!("{}{}", explain, "...");
    }

    pub(crate) fn set_offset(&mut self, offset: i32) {
        let new_offset = (self.backlog_offset as i32 + offset).max(0);
        // 不能超过最大可偏移量
        let max_offset = self.max_offset();
        self.backlog_offset = new_offset.min(max_offset as i32) as usize;
    }

    fn max_offset(&self) -> usize {
        self.backlog.len().saturating_sub(WINDOW_SIZE)
    }

    pub(crate) fn set_current_bgm(&mut self, bgm: String) {
        self.current_bgm = bgm;
    }

    pub(crate) fn set_pre_voice(&mut self, pre_voice: (SharedString, SharedString)) {
        self.pre_voice = Some(pre_voice);
    }

    pub(crate) fn set_pre_items(&mut self, jump_index: Option<usize>) {
        if let Some(index) = jump_index {
            let pre_bgm = match self.bgm.range(..=index).next_back() {
                Some((_, bgm)) => {
                    if &self.current_bgm != bgm {
                        Play(bgm.to_string())
                    } else {
                        PreBgm::None
                    }
                }
                None => PreBgm::Stop,
            };
            let pre_bg = self
                .backgrounds
                .range(..=index)
                .next_back()
                .map(|(_, bg)| bg.clone());
            let pre_figures = self
                .figures
                .range(..=index)
                .next_back()
                .map(|(_, fg)| fg.clone());
            self.pre_items = PreItems {
                pre_bg,
                pre_bgm,
                pre_figures,
            };
            self.current_block = index;
        }
    }

    pub(crate) fn update_figures(
        &mut self,
        index: usize,
        distance: &str,
        position: &str,
        command: Command,
    ) {
        self.figures
            .entry(index)
            .or_default()
            .push(distance, position, command);
    }

    pub(crate) fn set_backlog(&mut self, backlog: Vec<BackLogItem>) {
        self.backlog = backlog;
    }

    pub(crate) fn insert_background(&mut self, index: usize, command: Command) {
        self.backgrounds.insert(index, command);
    }

    pub(crate) fn insert_bgm(&mut self, index: usize, bgm: String) {
        self.bgm.insert(index, bgm);
    }

    pub(crate) fn insert_choice(&mut self, choice: String, label: Label) {
        self.choices.insert(choice, label);
    }

    pub(crate) fn insert_clear(&mut self, index: usize) {
        self.clear.insert(index);
    }

    pub(crate) fn insert_label(&mut self, label: String, index: usize) {
        self.labels.insert(label, index);
    }

    pub(crate) fn push_backlog(
        &mut self,
        name: SharedString,
        text: SharedString,
        voice: Option<(SharedString, SharedString)>,
    ) {
        let (chara, voice) = voice.unwrap_or_default();
        self.backlog.push(BackLogItem {
            front: name,
            back: text,
            script: self.name.to_shared_string(),
            index: self.current_block as i32,
            chara,
            voice,
        });
    }

    pub(crate) fn push_command(&mut self, command: Commands) {
        self.commands.push(command);
    }

    pub(crate) fn backlog(&self) -> Vec<BackLogItem> {
        let total = self.backlog.len();
        if total == 0 {
            return vec![];
        }

        let end = total.saturating_sub(self.backlog_offset);
        let start = end.saturating_sub(WINDOW_SIZE);
        self.backlog[start..end].to_vec()
    }

    pub(crate) fn last_voice(&self) -> Option<(String, String)> {
        let backlog = self.backlog.last().unwrap();
        if backlog.voice.is_empty() && backlog.chara.is_empty() {
            return None;
        }
        Some((backlog.chara.to_string(), backlog.voice.to_string()))
    }

    pub(crate) fn take_backlog(self) -> Vec<BackLogItem> {
        self.backlog
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn index(&self) -> usize {
        self.current_block
    }

    pub(crate) fn explain(&self) -> &str {
        &self.explain
    }

    pub(crate) fn current_bgm(&self) -> &str {
        &self.current_bgm
    }

    pub(crate) fn pre_voice(&mut self) -> Option<(SharedString, SharedString)> {
        self.pre_voice.take()
    }

    pub(crate) fn pre_items(&mut self) -> (Option<Command>, PreBgm, Option<Figure>) {
        let pre_items = self.pre_items.clone();
        self.pre_items = PreItems::default();
        (pre_items.pre_bg, pre_items.pre_bgm, pre_items.pre_figures)
    }

    pub(crate) fn find_label(&self, name: &str) -> Option<&usize> {
        self.labels.get(name)
    }

    pub(crate) fn get_choice_label(&self, name: &str) -> Option<&Label> {
        self.choices.get(name)
    }

    pub(crate) fn change_figure(
        &mut self,
        index: usize,
        distance: &str,
        position: &str,
    ) -> Command {
        let pos = format!("{distance}{position}");
        let mut idx = 0;
        for i in (0..=index).rev() {
            if let Some(fg) = self.figures.get(&i) {
                if fg.0.contains_key(&pos) {
                    idx = i;
                    break;
                }
            }
        }
        let figure = self.figures.get_mut(&idx).unwrap();
        figure.0.remove(&pos).unwrap()
    }

    pub(crate) fn in_clear(&self) -> bool {
        self.clear.contains(&self.current_block)
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct Figure(pub(crate) HashMap<String, Command>);

impl Figure {
    fn push(&mut self, distance: &str, position: &str, command: Command) {
        self.0.insert(format!("{distance}{position}"), command);
    }
}

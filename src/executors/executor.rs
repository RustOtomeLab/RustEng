use crate::config::{
    cg::CG_LENGTH, extra::save_extra_config, figure::FIGURE_CONFIG, save_load::SaveData,
    user::save_user_config, voice::VOICE_LENGTH, ENGINE_CONFIG,
};
use crate::error::{EngineError, ExecutorError, SaveError};
use crate::executors::{
    delay_executor::{DelayChannels, DelayTX},
    text_executor::{DisplayText, TextTX},
};
use crate::media::{
    player::{MediaPlayer, PreBgm, PreBgm::Play},
    video_player::{VideoContext, VideoPlayer},
};
use crate::parser::script_parser::{Command, Commands};
use crate::script::{Label, Script};
use crate::ui::initialize::{CharacterVolume, ExItem, FigureItem, MainWindow, SaveItem};
use slint::{Image, Model, SharedString, ToSharedString, VecModel, Weak};
use std::{
    cell::RefCell,
    fs,
    path::Path,
    rc::Rc,
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::sync::mpsc::Sender;

pub(crate) enum Jump {
    Label(Label),
    Index((String, i32)),
}

fn parse_position(position: &str, distance: &str) -> (f32, f32, f32) {
    let (width_ratio, default_base_y) = match distance {
        "z1" => (0.34, 0.125),   // 1/8
        "no" => (0.25, 0.16667), // 1/6
        _ => (0.34, 0.125),
    };

    let (base_x, base_y) = if position.starts_with('(') && position.ends_with(')') {
        let inner = &position[1..position.len() - 1];
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 2 {
            (
                parts[0].trim().parse().unwrap_or(0.33),
                parts[1].trim().parse().unwrap_or(default_base_y),
            )
        } else {
            (0.33, default_base_y)
        }
    } else {
        let bx = match position {
            "-2" | "vl" => 0.16,
            "-1" | "sl" => 0.0,
            "0" | "m" => 0.33,
            "1" | "sr" => 0.66,
            "2" | "vr" => 0.5,
            _ => 0.33,
        };
        if distance == "no" {
            (0.375, default_base_y)
        } else {
            (bx, default_base_y)
        }
    };

    (base_x, base_y, width_ratio)
}

#[derive(Clone)]
pub(crate) struct Executor {
    script: Rc<RefCell<Script>>,
    media_player: Rc<RefCell<MediaPlayer>>,
    cg: Rc<RefCell<u64>>,
    weak: Weak<MainWindow>,
    text: Arc<RwLock<DisplayText>>,
    choose_lock: Rc<RefCell<bool>>,
    video_context: Rc<RefCell<VideoContext>>,
    figure_items: Rc<VecModel<FigureItem>>,
    figure_id: Rc<RefCell<i32>>,
    text_tx: Option<TextTX>,
    auto_tx: Option<Sender<Duration>>,
    delay_channels: Option<DelayChannels>,
}

impl Executor {
    pub(crate) fn new(weak: Weak<MainWindow>) -> Result<Executor, EngineError> {
        let mut script = Script::new();
        script.with_name("ky01")?;

        let figure_items = Rc::new(VecModel::<FigureItem>::default());
        if let Some(window) = weak.upgrade() {
            window.set_figure_items(figure_items.clone().into());
        }

        Ok(Executor {
            script: Rc::new(RefCell::new(script)),
            media_player: Rc::new(RefCell::new(MediaPlayer::new()?)),
            cg: Rc::new(RefCell::new(0)),
            weak,
            text: Arc::new(RwLock::new(DisplayText::new())),
            choose_lock: Rc::new(RefCell::new(false)),
            video_context: Rc::new(RefCell::new(VideoContext::default())),
            figure_items,
            figure_id: Rc::new(RefCell::new(0)),
            text_tx: None,
            auto_tx: None,
            delay_channels: None,
        })
    }

    pub(crate) fn get_weak(&self) -> Weak<MainWindow> {
        self.weak.clone()
    }

    pub(crate) fn set_cg(&mut self, cg: u64) {
        *self.cg.borrow_mut() = cg;
    }

    pub(crate) fn set_text_tx(&mut self, text_tx: Sender<Arc<RwLock<DisplayText>>>) {
        self.text_tx = Some(text_tx);
    }

    pub(crate) fn set_auto_tx(&mut self, auto_tx: Sender<Duration>) {
        self.auto_tx = Some(auto_tx);
    }

    pub(crate) fn set_delay_channels(
        &mut self,
        delay_tx: DelayTX,
        delay_move_tx: DelayTX,
        loop_move_tx: DelayTX,
    ) {
        self.delay_channels = Some(DelayChannels {
            delay_tx,
            delay_move_tx,
            loop_move_tx,
        });
    }

    pub(crate) fn unlock(&mut self, index: usize) {
        let mut cg = self.cg.borrow_mut();
        *cg |= 1u64 << index;
    }

    pub(crate) fn execute_backlog(&self) -> Result<(), EngineError> {
        if let Some(window) = self.weak.upgrade() {
            let script = self.script.borrow();
            let backlog = script.backlog();
            window.set_backlogs(Rc::new(VecModel::from(backlog)).into());
        }

        Ok(())
    }

    pub(crate) fn execute_backlog_change(&mut self, offset: i32) -> Result<(), EngineError> {
        {
            let mut script = self.script.borrow_mut();
            script.set_offset(offset);
        }
        self.execute_backlog()
    }

    pub(crate) fn execute_backlog_jump(
        &mut self,
        name: String,
        index: i32,
    ) -> Result<(), EngineError> {
        if let Some(window) = self.weak.upgrade() {
            window.set_is_backlog(false);
        }
        self.execute_load(name, index)
    }

    pub(crate) fn execute_replay(&mut self) -> Result<(), EngineError> {
        let script = self.script.borrow();
        if let Some((name, voice)) = script.last_voice() {
            self.play_voice(&name, &voice)?;
        }
        Ok(())
    }

    pub(crate) fn execute_save(&mut self, index: i32) -> Result<(), EngineError> {
        if let Some(window) = self.weak.upgrade() {
            let script = self.script.borrow();
            let bg = window.get_bg();
            let exists_save_items = window.get_save_items();
            exists_save_items.set_row_data(
                index as usize,
                SaveItem {
                    bg: bg.0.clone(),
                    explain: SharedString::from(script.explain()),
                    index: script.index() as i32,
                    name: SharedString::from(script.name()),
                },
            );
            fs::write(
                format!("{}{}.toml", ENGINE_CONFIG.save_path(), index),
                toml::to_string_pretty(&SaveData::new(
                    script.name().to_string(),
                    script.index(),
                    script.explain().to_string(),
                    bg.0.path().unwrap().to_str().unwrap().to_string(),
                ))
                .map_err(SaveError::from)?,
            )
            .map_err(|e| SaveError::Write {
                path: format!("{}{}.toml", ENGINE_CONFIG.save_path(), index),
                source: e,
            })?;
            window.set_save_items(exists_save_items);
        }

        Ok(())
    }

    pub(crate) fn execute_load(&mut self, name: String, index: i32) -> Result<(), EngineError> {
        if !name.is_empty() {
            let weak = self.weak.clone();
            if let Some(window) = weak.upgrade() {
                window.set_current_screen(2);
                window.set_current_choose(0);
            }
            self.execute_jump(Jump::Index((name, index - 1)))?;
            self.execute_script()?;
        }

        Ok(())
    }

    pub(crate) fn execute_get_ex(&self) -> Result<(), EngineError> {
        let ex_items = Rc::new(VecModel::from(Vec::new()));

        let cgs = *self.cg.borrow();
        let mut i = 1;
        while i <= 63 {
            if cgs & (1 << i) != 0 {
                if let Some((_, length)) = CG_LENGTH.find_by_id(i) {
                    let (mut images, mut l, is_lock) = (Vec::new(), *length, false);
                    for j in 1..=*length {
                        if cgs & (1 << (j + i - 1)) != 0 {
                            if let Some((name, _)) = CG_LENGTH.find_by_id(j + i - 1) {
                                images.push(
                                    Image::load_from_path(Path::new(&format!(
                                        "{}{}.png",
                                        ENGINE_CONFIG.cg_path(),
                                        name
                                    )))
                                    .unwrap(),
                                );
                            } else {
                                return Err(ExecutorError::CgMetadataMissing(j + i - 1).into());
                            }
                        } else {
                            l -= 1;
                        }
                    }
                    i += *length;
                    ex_items.push(ExItem {
                        bg: Rc::new(VecModel::from(images)).into(),
                        indexs: l as i32,
                        is_lock,
                    })
                } else {
                    return Err(ExecutorError::CgMetadataMissing(i).into());
                }
            } else {
                i += 1;
                ex_items.push(ExItem::default())
            }
        }

        if let Some(window) = self.weak.upgrade() {
            window.set_ex_items(ex_items.into());
        }

        Ok(())
    }

    pub(crate) fn execute_bgm_volume(&mut self) -> Result<(), EngineError> {
        if let Some(window) = self.weak.upgrade() {
            let volume = window.get_main_volume() / 100.0;
            let bgm_volume = window.get_bgm_volume() / 100.0;
            self.media_player
                .borrow()
                .change_bgm_volume(volume * bgm_volume);
        }

        Ok(())
    }

    pub(crate) fn execute_voice_volume(&mut self) -> Result<(), EngineError> {
        if let Some(window) = self.weak.upgrade() {
            let volume = window.get_main_volume() / 100.0;
            let voice_volume = window.get_voice_volume() / 100.0;
            self.media_player
                .borrow()
                .change_voice_volume(volume * voice_volume);
        }

        Ok(())
    }

    pub(crate) fn execute_save_config(&self) -> Result<(), EngineError> {
        let weak = self.get_weak();
        save_user_config(weak)
    }

    pub(crate) fn execute_choose(&mut self, choice: SharedString) -> Result<(), EngineError> {
        *self.choose_lock.borrow_mut() = false;

        let label: (String, String);
        {
            let scr = self.script.clone();
            let scr = scr.borrow();
            label = scr.get_choice_label(&choice).unwrap().clone();
        }

        if let Some(window) = self.weak.upgrade() {
            window.set_choose_branch(Rc::new(VecModel::from(vec![])).into());
            window.set_current_choose(0);
            window.set_speaker("".into());
            window.set_dialogue_1(choice);
            window.set_dialogue_2(SharedString::default());
            window.set_dialogue_3(SharedString::default());
        }

        if let Some(window) = self.weak.upgrade() {
            if window.get_is_auto() {
                self.auto_tx
                    .clone()
                    .unwrap()
                    .try_send(Duration::from_secs(5))?;
            }
        }
        self.execute_jump(Jump::Label(label))
    }

    pub(crate) fn execute_jump(&mut self, label: Jump) -> Result<(), EngineError> {
        {
            let mut script = self.script.borrow_mut();
            let current_bgm = script.current_bgm().to_string();
            let mut pre_bgm = PreBgm::None;
            let backlog = script.to_owned().take_backlog();
            let jump_index = match label {
                Jump::Label((name, label)) => {
                    if name != script.name() {
                        let mut scr = Script::new();
                        scr.with_name(&name)?;
                        scr.set_backlog(backlog);
                        *script = scr;
                    }
                    script.find_label(&label).copied()
                }
                Jump::Index((name, index)) => {
                    if name != script.name() {
                        let mut scr = Script::new();
                        scr.with_name(&name)?;
                        scr.set_backlog(backlog);
                        *script = scr;
                    }
                    Some(index as usize)
                }
            };

            let mut current_block = script.index();
            if let Some(index) = jump_index {
                current_block = index;
                if let Some((_, bgm)) = script.get_bgm(index) {
                    if &current_bgm != bgm {
                        pre_bgm = Play(bgm.to_string());
                    }
                } else if script.get_bgm(index).is_none() {
                    pre_bgm = PreBgm::Stop;
                }
                {
                    let pre_bg = script.get_background(index).map(|(_, bg)| bg.clone());
                    script.set_pre_bg(pre_bg);
                    let pre_figures = script.get_figures(index).map(|(_, fg)| fg.clone());
                    script.set_pre_figures(pre_figures);
                }
            }
            script.set_pre_bgm(pre_bgm);
            script.set_index(current_block);
        }
        self.clean_fg("All")?;

        Ok(())
    }

    pub(crate) fn execute_auto(&mut self, tx: Sender<()>, source: bool) -> Result<(), EngineError> {
        if let Some(window) = self.weak.upgrade() {
            if source {
                self.auto_tx
                    .clone()
                    .unwrap()
                    .try_send(Duration::from_secs(1))?;
                tx.try_send(())?;
            } else {
                if window.get_is_auto() {
                    tx.try_send(())?;
                }
                window.set_is_auto(false);
            }
        }

        Ok(())
    }

    pub(crate) fn execute_skip(&mut self, tx: Sender<()>, source: bool) -> Result<(), EngineError> {
        if let Some(window) = self.weak.upgrade() {
            if source {
                tx.try_send(())?;
            } else {
                if window.get_is_skip() {
                    tx.try_send(())?;
                }
                window.set_is_skip(false);
            }
        }

        Ok(())
    }

    pub(crate) fn execute_script(&mut self) -> Result<(), EngineError> {
        {
            let scr = self.script.clone();
            let scr = scr.borrow();
            if scr.in_clear() {
                self.delay_channels.as_ref().unwrap().clear_all();
            } else {
                self.delay_channels.as_ref().unwrap().skip_all();
            }
        }

        let res = {
            let mut text = self.text.write().unwrap();
            if text.is_running {
                text.end();
                true
            } else {
                false
            }
        };
        if res {
            if let Some(window) = self.weak.upgrade() {
                if window.get_is_auto() {
                    self.auto_tx
                        .clone()
                        .unwrap()
                        .try_send(Duration::from_secs(2))?;
                }
            }
            return Ok(());
        }

        let mut duration = Duration::default();
        let mut is_wait = true;
        let mut is_auto = false;
        if let Some(window) = self.weak.upgrade() {
            duration += Duration::from_millis((window.get_delay() * 1000.0) as u64);
            is_wait = window.get_is_wait();
            is_auto = window.get_is_auto();
        }

        if *self.choose_lock.borrow() {
            return Ok(());
        }

        if self.video_context.try_borrow().is_err() {
            return Ok(());
        }

        let mut commands = Commands::EmptyCmd;
        {
            let scr = self.script.clone();
            let mut scr = scr.borrow_mut();
            if let Some(cmds) = scr.next_command() {
                commands = cmds.clone();
            }
        }
        let delay = match commands {
            Commands::EmptyCmd => unreachable!(),
            Commands::OneCmd(command) => self.apply_command(command)?,
            Commands::VarCmds(vars) => {
                let mut delay = Duration::default();
                for command in vars {
                    delay += self.apply_command(command)?;
                }
                delay
            }
        };

        if is_wait {
            duration += delay;
        }

        if is_auto {
            self.auto_tx.clone().unwrap().try_send(duration)?;
        }

        Ok(())
    }

    pub(crate) fn apply_command(&mut self, command: Command) -> Result<Duration, EngineError> {
        let mut duration = Duration::from_secs(0);

        if let Some(window) = self.weak.upgrade() {
            let pre_bg;
            let pre_bgm;
            let pre_fg;

            {
                let mut scr = self.script.borrow_mut();
                pre_bg = scr.pre_bg();
                pre_bgm = scr.pre_bgm();
                pre_fg = scr.pre_figures();
            }

            if let Some(bg) = pre_bg {
                self.show_bg(&bg)?;
            }
            if let Play(bgm) = pre_bgm {
                self.play_bgm(bgm)?;
            } else if let PreBgm::Stop = pre_bgm {
                self.media_player.borrow().stop_bgm();
            }
            if let Some(figures) = pre_fg {
                for figure in figures.0.values() {
                    self.show_fg(&figure.clone())?;
                }
            }

            match command {
                Command::Background { .. } => self.show_bg(&command)?,
                Command::PlayBgm(bgm) => {
                    let needs_play = {
                        let mut script = self.script.borrow_mut();
                        if bgm != script.current_bgm() {
                            script.set_current_bgm(bgm.clone());
                            true
                        } else {
                            false
                        }
                    };
                    if needs_play {
                        self.play_bgm(bgm)?;
                    }
                }
                Command::Choice((explain, choices)) => {
                    *self.choose_lock.borrow_mut() = true;

                    let mut script = self.script.borrow_mut();
                    script.set_explain(&format!("选择支：{explain}"));
                    let mut choose_branch = Vec::with_capacity(choices.len());
                    for (index, choice) in choices.iter().enumerate() {
                        choose_branch.push((index as i32, SharedString::from(choice.0.clone())));
                    }
                    script.push_backlog(
                        "选择支".to_shared_string(),
                        explain.to_shared_string(),
                        None,
                    );
                    window.set_choose_branch(Rc::new(VecModel::from(choose_branch)).into());
                    window.set_current_choose(choices.len() as i32);
                }
                Command::Dialogue { speaker, text } => {
                    {
                        let mut script = self.script.borrow_mut();
                        let voice = script.pre_voice();
                        script.set_explain(&text);
                        script.push_backlog(
                            speaker.to_shared_string(),
                            text.replace("{nns}", "").to_shared_string(),
                            voice,
                        );
                    }
                    window.set_speaker(SharedString::from(speaker));
                    {
                        let mut send_text = self.text.write().unwrap();
                        send_text.start_animation(text, window.get_text_speed());
                    }
                    let tx = self.text_tx.clone().unwrap();
                    tx.try_send(self.text.clone())?;
                }
                Command::PlayVoice {
                    ref name,
                    ref voice,
                } => {
                    let mut script = self.script.borrow_mut();
                    script.set_pre_voice((name.to_shared_string(), voice.to_shared_string()));
                    duration += self.play_voice(name, voice)?;
                }
                Command::PlayVideo(name) => {
                    self.start_video(&name)?;
                }
                Command::Figure { .. } => {
                    self.show_fg(&command)?;
                }
                Command::Move { .. } => {
                    self.show_move(&command)?;
                }
                Command::Clear(name) => self.clean_fg(&name)?,
                Command::Jump(jump) => {
                    self.execute_jump(Jump::Label(jump))?;
                }
                Command::Label => (),
            }
        };

        Ok(duration)
    }

    fn play_bgm(&self, bgm: String) -> Result<(), EngineError> {
        let weak = self.weak.clone();

        if let Some(window) = weak.upgrade() {
            let volume = window.get_main_volume() / 100.0;
            let bgm_volume = window.get_bgm_volume() / 100.0;
            self.media_player.borrow().play_bgm(
                &format!("{}{}.ogg", ENGINE_CONFIG.bgm_path(), bgm),
                volume * bgm_volume,
            )?;
        }

        Ok(())
    }

    pub(crate) fn play_voice(
        &self,
        name: &String,
        voice: &String,
    ) -> Result<Duration, EngineError> {
        let weak = self.weak.clone();

        if let (Some(length), Some(window)) = (VOICE_LENGTH.find(name), weak.upgrade()) {
            let volume = window.get_main_volume() / 100.0;
            let voice_volume = window.get_voice_volume() / 100.0;
            let character_volumes = window.get_character_volumes();
            {
                let full_name = ENGINE_CONFIG.character_list().get(name).unwrap();
                for CharacterVolume {
                    name: ch_name,
                    volume: ch_volume,
                } in character_volumes.iter()
                {
                    if ch_name == full_name {
                        self.media_player.borrow().play_voice(
                            &format!("{}/{}/{}.ogg", ENGINE_CONFIG.voice_path(), name, voice),
                            volume * voice_volume * ch_volume / 100.0,
                        )?;
                        break;
                    }
                }
            }
            return Ok(*length.get(voice).unwrap());
        }

        Ok(Duration::from_secs(0))
    }

    fn show_bg(&mut self, bg: &Command) -> Result<(), EngineError> {
        let weak = self.weak.clone();
        let Command::Background {
            name,
            x_offset,
            y_offset,
            zoom,
            is_cg,
        } = bg
        else {
            unreachable!()
        };

        let path = if *is_cg {
            if let Some((index, _)) = CG_LENGTH.find_by_name(name) {
                self.unlock(*index);
                save_extra_config(*self.cg.borrow())?;
            }
            ENGINE_CONFIG.cg_path()
        } else {
            ENGINE_CONFIG.background_path()
        };

        if let Some(window) = weak.upgrade() {
            let image = Image::load_from_path(Path::new(&format!("{path}{name}.png"))).unwrap();
            window.set_bg((
                image,
                x_offset.unwrap_or(0.0),
                y_offset.unwrap_or(0.0),
                zoom.unwrap_or(1.0),
            ));
        }

        Ok(())
    }

    pub(crate) fn show_fg(&self, fg: &Command) -> Result<(), EngineError> {
        let Command::Figure {
            name,
            distance,
            body,
            face,
            position,
            delay,
        } = fg
        else {
            unreachable!()
        };

        if delay.is_some() {
            self.delay_channels.as_ref().unwrap().send_delay(fg)?;
            return Ok(());
        }

        if let (Some(body_para), Some(face_para), Some(offset)) = FIGURE_CONFIG.find(name) {
            let rate = *body_para.get(body).unwrap();
            let body_img = Image::load_from_path(Path::new(&format!(
                "{}{}/{}/{}.png",
                ENGINE_CONFIG.figure_path(),
                name,
                distance,
                body
            )))
            .unwrap();
            let (face_x, face_y) = face_para.get(face).unwrap();
            let face_img = Image::load_from_path(Path::new(&format!(
                "{}{}/{}/{}.png",
                ENGINE_CONFIG.figure_path(),
                name,
                distance,
                face
            )))
            .unwrap();

            let (base_x, base_y, width_ratio) = parse_position(position, distance);

            let model = self.figure_items.clone();
            let mut found_idx = None;
            for i in 0..model.row_count() {
                let item = model.row_data(i).unwrap();
                if item.name == name {
                    found_idx = Some(i);
                    break;
                }
            }

            let id = if let Some(i) = found_idx {
                model.row_data(i).unwrap().id
            } else {
                let mut id_counter = self.figure_id.borrow_mut();
                *id_counter += 1;
                *id_counter
            };

            let item = FigureItem {
                id,
                name: name.to_shared_string(),
                distance: distance.to_shared_string(),
                body: body_img,
                face: face_img,
                rate,
                offset: *offset,
                face_x: *face_x,
                face_y: *face_y,
                base_x,
                base_y,
                x_offset: 0.0,
                y_offset: 0.0,
                width_ratio,
            };

            if let Some(i) = found_idx {
                model.set_row_data(i, item);
            } else {
                model.push(item);
            }
        }

        Ok(())
    }

    pub(crate) fn show_move(&self, fg_move: &Command) -> Result<(), EngineError> {
        let Command::Move {
            name,
            distance,
            position,
            action,
            repeat,
            delay,
        } = fg_move
        else {
            unreachable!()
        };

        if delay.is_some() {
            self.delay_channels.as_ref().unwrap().send_delay(fg_move)?;
            return Ok(());
        }

        let container_height = if let Some(window) = self.weak.upgrade() {
            window.get_container_height()
        } else {
            0.0
        };

        match &action[..] {
            "to2" | "to0" => {
                let target_pos = if action == "to2" { "2" } else { "0" };
                if *repeat != 1 {
                    let back = fg_move.back();
                    let next = Command::Move {
                        name: name.to_string(),
                        distance: distance.to_string(),
                        position: position.to_string(),
                        action: action.clone(),
                        repeat: if *repeat > 1 { *repeat - 1 } else { -1 },
                        delay: Some("301".to_string()),
                    };
                    self.delay_channels.as_ref().unwrap().send_loop(next, back);
                }
                self.move_figure(name, target_pos, distance)?;
            }
            "nod" => {
                if *repeat != 1 {
                    let back = fg_move.back();
                    let nod = Command::Move {
                        name: name.to_string(),
                        distance: distance.to_string(),
                        position: position.to_string(),
                        action: "nod".to_string(),
                        repeat: if *repeat > 1 { *repeat - 1 } else { -1 },
                        delay: Some("301".to_string()),
                    };
                    self.delay_channels.as_ref().unwrap().send_loop(nod, back);
                } else {
                    self.delay_channels
                        .as_ref()
                        .unwrap()
                        .send_move(Command::Move {
                            name: name.to_string(),
                            distance: distance.to_string(),
                            position: position.to_string(),
                            action: "back".to_string(),
                            repeat: *repeat,
                            delay: Some("150".to_string()),
                        })?;
                }
                self.set_figure_offset(name, 0.0, container_height / 40.0)?;
            }
            "back" => {
                let (base_x, _, _) = parse_position(position, distance);
                let model = self.figure_items.clone();
                for i in 0..model.row_count() {
                    let item = model.row_data(i).unwrap();
                    if item.name == name {
                        let mut new_item = item.clone();
                        new_item.base_x = base_x;
                        new_item.x_offset = 0.0;
                        new_item.y_offset = 0.0;
                        model.set_row_data(i, new_item);
                        break;
                    }
                }
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    fn move_figure(&self, name: &str, target_pos: &str, distance: &str) -> Result<(), EngineError> {
        let (base_x, _, _) = parse_position(target_pos, distance);
        let model = self.figure_items.clone();
        for i in 0..model.row_count() {
            let item = model.row_data(i).unwrap();
            if item.name == name {
                let mut new_item = item.clone();
                new_item.base_x = base_x;
                new_item.x_offset = 0.0;
                model.set_row_data(i, new_item);
                break;
            }
        }
        Ok(())
    }

    fn set_figure_offset(
        &self,
        name: &str,
        x_offset: f32,
        y_offset: f32,
    ) -> Result<(), EngineError> {
        let model = self.figure_items.clone();
        for i in 0..model.row_count() {
            let item = model.row_data(i).unwrap();
            if item.name == name {
                let mut new_item = item.clone();
                new_item.x_offset = x_offset;
                new_item.y_offset = y_offset;
                model.set_row_data(i, new_item);
                break;
            }
        }
        Ok(())
    }

    fn clean_fg(&self, target: &str) -> Result<(), EngineError> {
        let model = self.figure_items.clone();

        if target == "All" {
            while model.row_count() > 0 {
                model.remove(0);
            }
        } else {
            // 从后往前删，避免索引错位
            let mut i = model.row_count();
            while i > 0 {
                i -= 1;
                let item = model.row_data(i).unwrap();
                if item.name == target {
                    model.remove(i);
                }
            }
        }

        Ok(())
    }

    fn start_video(&self, name: &str) -> Result<(), EngineError> {
        let path = format!(
            "{}{}.{}",
            ENGINE_CONFIG.video_path(),
            name,
            ENGINE_CONFIG.video_extension()
        );

        self.media_player.borrow().stop_all();

        let player = VideoPlayer::play(&path)?;
        let mut video_context = self.video_context.borrow_mut();
        video_context.set_video_player(player);

        if let Some(window) = self.weak.upgrade() {
            window.set_is_video(true);
        }

        let timer = slint::Timer::default();
        let weak = self.weak.clone();
        let video_player = self.video_context.clone();
        let executor_for_finish = self.clone();
        timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(16),
            move || {
                let mut finished = false;
                if let Some(vp) = video_player.borrow().get_video_player_ref() {
                    if let Some(window) = weak.upgrade() {
                        if let Some(frame) = vp.take_latest_frame() {
                            window.set_video_frame(frame);
                        }
                    }
                    finished = vp.is_finished();
                }
                if finished {
                    let executor = executor_for_finish.clone();
                    slint::spawn_local(async move {
                        if let Err(e) = executor.execute_stop_video().await {
                            eprintln!("video auto-stop failed: {e}");
                        }
                    })
                    .expect("video timer: no slint event loop");
                }
            },
        );
        video_context.set_video_timer(timer);

        Ok(())
    }

    pub(crate) async fn execute_stop_video(&self) -> Result<(), EngineError> {
        {
            let mut video_context = self.video_context.borrow_mut();

            if let Some(player) = video_context.get_video_player() {
                player.stop();
            } else {
                return Ok(());
            }

            video_context.get_video_timer();
        }

        if let Some(window) = self.weak.upgrade() {
            window.set_is_video(false);
            window.set_video_frame(Image::default());
        }

        let mut this = self.clone();
        this.execute_script()
    }
}

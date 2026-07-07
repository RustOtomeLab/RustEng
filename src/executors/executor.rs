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
use crate::ui::initialize::{CharacterVolume, MainWindow, SaveItem};
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

fn figure_default() -> (Image, f32, f32) {
    (Image::default(), 0.0, 0.0)
}

fn face_default() -> (Image, f32, f32) {
    (Image::default(), 0.0, 0.0)
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
    text_tx: Option<TextTX>,
    auto_tx: Option<Sender<Duration>>,
    delay_channels: Option<DelayChannels>,
}

impl Executor {
    pub(crate) fn new(weak: Weak<MainWindow>) -> Result<Executor, EngineError> {
        let mut script = Script::new();
        script.with_name("ky01")?;

        Ok(Executor {
            script: Rc::new(RefCell::new(script)),
            media_player: Rc::new(RefCell::new(MediaPlayer::new()?)),
            cg: Rc::new(RefCell::new(0)),
            weak,
            text: Arc::new(RwLock::new(DisplayText::new())),
            choose_lock: Rc::new(RefCell::new(false)),
            video_context: Rc::new(RefCell::new(VideoContext::default())),
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
        let mut ex_items = Vec::with_capacity(16);

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
                    ex_items.push((Rc::new(VecModel::from(images)).into(), l as i32, is_lock))
                } else {
                    return Err(ExecutorError::CgMetadataMissing(i).into());
                }
            } else {
                i += 1;
                ex_items.push((
                    Rc::new(VecModel::from(vec![Image::default()])).into(),
                    0,
                    true,
                ))
            }
        }

        if let Some(window) = self.weak.upgrade() {
            window.set_ex_items(Rc::new(VecModel::from(ex_items)).into());
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
        save_user_config(weak)?;

        Ok(())
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
        self.clean_fg("All", "All")?;

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
                    script.push_backlog("选择支".to_shared_string(), explain.to_shared_string());
                    window.set_choose_branch(Rc::new(VecModel::from(choose_branch)).into());
                    window.set_current_choose(choices.len() as i32);
                }
                Command::Dialogue { speaker, text } => {
                    {
                        let mut script = self.script.borrow_mut();
                        script.set_explain(&text);
                        script.push_backlog(speaker.to_shared_string(), text.to_shared_string());
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
                    if let Some(length) = VOICE_LENGTH.find(name) {
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
                                        &format!(
                                            "{}/{}/{}.ogg",
                                            ENGINE_CONFIG.voice_path(),
                                            name,
                                            voice
                                        ),
                                        volume * voice_volume * ch_volume / 100.0,
                                    )?;
                                    break;
                                }
                            }
                        }
                        duration += *length.get(voice).unwrap();
                    }
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
                Command::Clear(distance, position) => self.clean_fg(&distance, &position)?,
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
        let weak = self.weak.clone();
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

        if let Some(window) = weak.upgrade() {
            if delay.is_some() {
                self.delay_channels.as_ref().unwrap().send_delay(fg)?;
                return Ok(());
            }
            if let (Some(body_para), Some(face_para), Some(offset)) = FIGURE_CONFIG.find(name) {
                let body_exist = match (&position[..], &distance[..]) {
                    ("-1", "z1") => window.get_fg_z1__1(),
                    ("-2", "z1") => window.get_fg_z1__2(),
                    ("0", "z1") => window.get_fg_z1_0(),
                    ("2", "z1") => window.get_fg_z1_2(),
                    ("1", "z1") => window.get_fg_z1_1(),
                    ("0", "no") => window.get_fg_no_0(),
                    _ => unreachable!(),
                }
                .0
                .path()
                .unwrap_or(Path::new(""))
                .to_str()
                .unwrap()
                .to_string();
                let ready_body = format!(
                    "{}{}/{}/{}.png",
                    ENGINE_CONFIG.figure_path(),
                    name,
                    distance,
                    body
                );

                if body_exist != ready_body {
                    let rate = body_para.get(body).unwrap();
                    let body = Image::load_from_path(Path::new(&ready_body)).unwrap();
                    match (&position[..], &distance[..]) {
                        ("-1", "z1") => window.set_fg_z1__1((body, *offset, *rate)),
                        ("-2", "z1") => window.set_fg_z1__2((body, *offset, *rate)),
                        ("0", "z1") => window.set_fg_z1_0((body, *offset, *rate)),
                        ("2", "z1") => window.set_fg_z1_2((body, *offset, *rate)),
                        ("1", "z1") => window.set_fg_z1_1((body, *offset, *rate)),
                        ("0", "no") => window.set_fg_no_0((body, *offset, *rate)),
                        _ => unreachable!(),
                    }
                }
                let (face_x, face_y) = face_para.get(face).unwrap();
                let face = Image::load_from_path(Path::new(&format!(
                    "{}{}/{}/{}.png",
                    ENGINE_CONFIG.figure_path(),
                    name,
                    distance,
                    face
                )))
                .unwrap();
                match (&position[..], &distance[..]) {
                    ("-1", "z1") => window.set_face_z1__1((face, *face_x, *face_y)),
                    ("-2", "z1") => window.set_face_z1__2((face, *face_x, *face_y)),
                    ("0", "z1") => window.set_face_z1_0((face, *face_x, *face_y)),
                    ("2", "z1") => window.set_face_z1_2((face, *face_x, *face_y)),
                    ("1", "z1") => window.set_face_z1_1((face, *face_x, *face_y)),
                    ("0", "no") => window.set_face_no_0((face, *face_x, *face_y)),
                    _ => unreachable!(),
                }
            }
        }

        Ok(())
    }

    pub(crate) fn show_move(&self, fg_move: &Command) -> Result<(), EngineError> {
        let weak = self.weak.clone();
        let Command::Move {
            name,
            distance,
            body,
            face,
            position,
            action,
            repeat,
            delay,
        } = fg_move
        else {
            unreachable!()
        };

        if let Some(window) = weak.upgrade() {
            if delay.is_some() {
                self.delay_channels.as_ref().unwrap().send_delay(fg_move)?;
                return Ok(());
            }

            let offset: (f32, f32) = match &action[..] {
                "to2" => {
                    if *repeat != 1 {
                        let back = fg_move.back();
                        let action = Command::Move {
                            name: name.to_string(),
                            distance: distance.to_string(),
                            body: body.to_string(),
                            face: face.to_string(),
                            position: position.to_string(),
                            action: "to2".to_string(),
                            repeat: if *repeat > 1 { *repeat - 1 } else { -1 },
                            delay: Some("301".to_string()),
                        };
                        self.delay_channels
                            .as_ref()
                            .unwrap()
                            .send_loop(action, back);
                    } else {
                        let tx = self.delay_channels.as_ref().unwrap();
                        tx.send_move(Command::Figure {
                            name: name.to_string(),
                            distance: distance.to_string(),
                            body: body.to_string(),
                            face: face.to_string(),
                            position: "2".to_string(),
                            delay: Some("150".to_string()),
                        })?;
                        tx.send_move(fg_move.back_and_clean())?;
                    }
                    match (&position[..], &distance[..]) {
                        ("-1", "z1") => (window.get_container_width() * 0.5, 0.0),
                        ("-2", "z1") => (window.get_container_width() * 0.34, 0.0),
                        ("0", "z1") => (window.get_container_width() * 0.17, 0.0),
                        ("1", "z1") => (-window.get_container_width() * 0.16, 0.0),
                        _ => unreachable!(),
                    }
                }
                "to0" => {
                    let tx = self.delay_channels.as_ref().unwrap();
                    tx.send_move(Command::Figure {
                        name: name.to_string(),
                        distance: distance.to_string(),
                        body: body.to_string(),
                        face: face.to_string(),
                        position: "0".to_string(),
                        delay: Some("150".to_string()),
                    })?;
                    tx.send_move(fg_move.back_and_clean())?;

                    match (&position[..], &distance[..]) {
                        ("-1", "z1") => (window.get_container_width() * 0.33, 0.0),
                        ("-2", "z1") => (window.get_container_width() * 0.17, 0.0),
                        ("2", "z1") => (-window.get_container_width() * 0.17, 0.0),
                        ("1", "z1") => (-window.get_container_width() * 0.33, 0.0),
                        _ => unreachable!(),
                    }
                }
                "nod" => {
                    if *repeat != 1 {
                        let back = fg_move.back();
                        let nod = Command::Move {
                            name: name.to_string(),
                            distance: distance.to_string(),
                            body: body.to_string(),
                            face: face.to_string(),
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
                                body: body.to_string(),
                                face: face.to_string(),
                                position: position.to_string(),
                                action: "back".to_string(),
                                repeat: *repeat,
                                delay: Some("150".to_string()),
                            })?;
                    }
                    (0.0, window.get_container_height() / 40.0)
                }
                "back" => (0.0, 0.0),
                "back_and_clean" => {
                    match (&position[..], &distance[..]) {
                        ("-1", "z1") => {
                            window.set_fg_z1__1(figure_default());
                            window.set_face_z1__1(face_default());
                        }
                        ("-2", "z1") => {
                            window.set_fg_z1__2(figure_default());
                            window.set_face_z1__2(face_default());
                        }
                        ("2", "z1") => {
                            window.set_fg_z1_2(figure_default());
                            window.set_face_z1_2(face_default());
                        }
                        ("0", "z1") => {
                            window.set_fg_z1_0(figure_default());
                            window.set_face_z1_0(face_default());
                        }
                        ("1", "z1") => {
                            window.set_fg_z1_1(figure_default());
                            window.set_face_z1_1(face_default());
                        }
                        _ => unreachable!(),
                    }
                    (0.0, 0.0)
                }
                _ => unreachable!(),
            };

            match (&position[..], &distance[..]) {
                ("-1", "z1") => window.set_offset_z1__1(offset),
                ("-2", "z1") => window.set_offset_z1__2(offset),
                ("0", "z1") => window.set_offset_z1_0(offset),
                ("2", "z1") => window.set_offset_z1_2(offset),
                ("1", "z1") => window.set_offset_z1_1(offset),
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    fn clean_fg(&self, distance: &str, position: &str) -> Result<(), EngineError> {
        let weak = self.weak.clone();
        if let Some(window) = weak.upgrade() {
            match (position, distance) {
                ("-2", "z1") => {
                    window.set_fg_z1__2(figure_default());
                    window.set_face_z1__2(face_default());
                }
                ("2", "z1") => {
                    window.set_fg_z1_2(figure_default());
                    window.set_face_z1_2(face_default());
                }
                ("0", "z1") => {
                    window.set_fg_z1_0(figure_default());
                    window.set_face_z1_0(face_default());
                }
                ("0", "no") => {
                    window.set_fg_no_0(figure_default());
                    window.set_face_no_0(face_default());
                }
                ("All", "All") => {
                    window.set_fg_z1__2(figure_default());
                    window.set_face_z1__2(face_default());
                    window.set_fg_z1_0(figure_default());
                    window.set_face_z1_2(face_default());
                    window.set_fg_z1_2(figure_default());
                    window.set_face_z1_0(face_default());
                    window.set_fg_no_0(figure_default());
                    window.set_face_no_0(face_default());
                }
                _ => unreachable!(),
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

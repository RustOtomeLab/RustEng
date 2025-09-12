use crate::audio::player::PreBgm::Play;
use crate::audio::player::{Player, PreBgm};
use crate::config::figure::FIGURE_CONFIG;
use crate::config::save_load::SaveData;
use crate::config::user::save_user_config;
use crate::config::voice::VOICE_CONFIG;
use crate::config::ENGINE_CONFIG;
use crate::error::EngineError;
use crate::executor::delay_executor::DelayTX;
use crate::executor::text_executor::DisplayText;
use crate::parser::parser::{Command, Commands};
use crate::script::{Label, Script};
use crate::ui::ui::MainWindow;
use slint::{Image, Model, SharedString, ToSharedString, VecModel, Weak};
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::time::Duration;
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

pub struct Executor {
    script: Rc<RefCell<Script>>,
    bgm_player: Rc<RefCell<Player>>,
    voice_player: Rc<RefCell<Player>>,
    weak: Weak<MainWindow>,
    text: Arc<RwLock<DisplayText>>,
    choose_lock: Rc<RefCell<bool>>,
    text_tx: Option<Sender<Arc<RwLock<DisplayText>>>>,
    auto_tx: Option<Sender<Duration>>,
    delay_tx: Option<DelayTX>,
    delay_move_tx: Option<DelayTX>,
    loop_move_tx: Option<DelayTX>,
}

impl Clone for Executor {
    fn clone(&self) -> Executor {
        Executor {
            script: self.script.clone(),
            bgm_player: self.bgm_player.clone(),
            voice_player: self.voice_player.clone(),
            weak: self.weak.clone(),
            text: self.text.clone(),
            choose_lock: self.choose_lock.clone(),
            text_tx: self.text_tx.clone(),
            auto_tx: self.auto_tx.clone(),
            delay_tx: self.delay_tx.clone(),
            delay_move_tx: self.delay_move_tx.clone(),
            loop_move_tx: self.loop_move_tx.clone(),
        }
    }
}

impl Executor {
    pub fn new(
        script: Rc<RefCell<Script>>,
        bgm_player: Rc<RefCell<Player>>,
        voice_player: Rc<RefCell<Player>>,
        weak: Weak<MainWindow>,
    ) -> Executor {
        Executor {
            script,
            bgm_player,
            voice_player,
            weak,
            text: Arc::new(RwLock::new(DisplayText::new())),
            choose_lock: Rc::new(RefCell::new(false)),
            text_tx: None,
            delay_tx: None,
            auto_tx: None,
            delay_move_tx: None,
            loop_move_tx: None,
        }
    }

    pub fn get_weak(&self) -> Weak<MainWindow> {
        self.weak.clone()
    }

    pub fn set_text_tx(&mut self, text_tx: Sender<Arc<RwLock<DisplayText>>>) {
        self.text_tx = Some(text_tx);
    }

    pub fn set_delay_tx(&mut self, delay_tx: DelayTX) {
        self.delay_tx = Some(delay_tx);
    }

    pub fn set_auto_tx(&mut self, auto_tx: Sender<Duration>) {
        self.auto_tx = Some(auto_tx);
    }

    pub fn set_delay_move_tx(&mut self, delay_move_tx: DelayTX) {
        self.delay_move_tx = Some(delay_move_tx);
    }

    pub fn set_loop_move_tx(&mut self, loop_move_tx: DelayTX) {
        self.loop_move_tx = Some(loop_move_tx);
    }

    pub async fn execute_backlog(&self) -> Result<(), EngineError> {
        if let Some(window) = self.weak.upgrade() {
            let script = self.script.borrow();
            //println!("{:#?}", script.backlog);
            let backlog = script.backlog();
            window.set_backlogs(Rc::new(VecModel::from(backlog)).into());
        }

        Ok(())
    }

    pub async fn execute_backlog_change(&mut self, offset: i32) -> Result<(), EngineError> {
        {
            let mut script = self.script.borrow_mut();
            script.set_offset(offset);
        }
        self.execute_backlog().await
    }

    pub async fn execute_backlog_jump(
        &mut self,
        name: String,
        index: i32,
    ) -> Result<(), EngineError> {
        if let Some(window) = self.weak.upgrade() {
            window.set_is_backlog(false);
        }
        self.execute_load(name, index).await
    }

    pub async fn execute_save(&mut self, index: i32) -> Result<(), EngineError> {
        if let Some(window) = self.weak.upgrade() {
            let script = self.script.borrow();
            let bg = window.get_bg();
            let mut save_items = Vec::with_capacity(16);
            let exists_save_items = window.get_save_items();
            for (i, item) in exists_save_items.iter().enumerate() {
                let mut sava_data = SaveData::new(
                    item.3.to_string(),
                    item.2 as usize,
                    item.1.to_string(),
                    item.0
                        .path()
                        .unwrap_or("".as_ref())
                        .to_str()
                        .unwrap()
                        .to_string(),
                );
                if i != index as usize {
                    save_items.push(item);
                } else {
                    sava_data = SaveData::new(
                        script.name.clone(),
                        script.index(),
                        script.explain().to_string(),
                        bg.path().unwrap().to_str().unwrap().to_string(),
                    );
                    save_items.push((
                        bg.clone(),
                        SharedString::from(script.explain()),
                        script.index() as i32,
                        SharedString::from(script.name()),
                    ));
                }
                fs::write(format!("{}{}.toml", ENGINE_CONFIG.save_path(), i), toml::to_string_pretty(&sava_data)?)?;
            }
            window.set_save_items(Rc::new(VecModel::from(save_items)).into());

            //println!("save {}", index);
        }

        Ok(())
    }

    pub async fn execute_load(&mut self, name: String, index: i32) -> Result<(), EngineError> {
        if !name.is_empty() {
            let weak = self.weak.clone();
            if let Some(window) = weak.upgrade() {
                window.set_current_screen(2);
                window.set_current_choose(0);
            }
            self.execute_jump(Jump::Index((name, index - 1))).await?;
            self.execute_script().await?;
        }

        Ok(())
    }

    pub async fn execute_bgm_volume(&mut self) -> Result<(), EngineError> {
        if let Some(window) = self.weak.upgrade() {
            let bgm_player = self.bgm_player.borrow_mut();
            let volume = window.get_main_volume() / 100.0;
            let bgm_volume = window.get_bgm_volume() / 100.0;
            bgm_player.change_volume(volume * bgm_volume);
        }

        Ok(())
    }

    pub async fn execute_voice_volume(&mut self) -> Result<(), EngineError> {
        if let Some(window) = self.weak.upgrade() {
            let voice_player = self.voice_player.borrow_mut();
            let volume = window.get_main_volume() / 100.0;
            let voice_volume = window.get_voice_volume() / 100.0;
            voice_player.change_volume(volume * voice_volume);
        }

        Ok(())
    }

    pub async fn execute_save_config(&self) -> Result<(), EngineError> {
        let weak = self.get_weak();
        save_user_config(weak)?;

        Ok(())
    }

    pub async fn execute_choose(&mut self, choice: SharedString) -> Result<(), EngineError> {
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
            window.set_dialogue(choice);
        }

        if let Some(window) = self.weak.upgrade() {
            if window.get_is_auto() {
                //println!("choose: 5s");
                self.auto_tx
                    .clone()
                    .unwrap()
                    .send(Duration::from_secs(5))
                    .await?;
            }
        }
        self.execute_jump(Jump::Label(label)).await
    }

    pub async fn execute_jump(&mut self, label: Jump) -> Result<(), EngineError> {
        let mut script = self.script.borrow_mut();
        let current_bgm = script.current_bgm().to_string();
        let mut pre_bg = None;
        let mut pre_bgm = PreBgm::None;
        let mut pre_figures = None;
        let backlog = script.to_owned().take_backlog();
        let jump_index = match label {
            Jump::Label((name, label)) => {
                if name != script.name() {
                    let mut scr = Script::new();
                    scr.with_name(&name)?;
                    scr.set_backlog(backlog);
                    *script = scr;
                }
                script.find_label(&label).map(|index| *index)
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
                pre_bg = script.get_background(index).map(|(_, bg)| bg.clone());
                pre_figures = script.get_figures(index).map(|(_, fg)| fg.clone());
            }
        }
        script.set_pre_bg(pre_bg);
        script.set_pre_bgm(pre_bgm);

        self.clean_fg("All", "All").await?;
        script.set_pre_figures(pre_figures);
        script.set_index(current_block);

        Ok(())
    }

    pub async fn execute_auto(&mut self, tx: Sender<()>, source: bool) -> Result<(), EngineError> {
        if let Some(window) = self.weak.upgrade() {
            if source {
                //println!("发送");
                self.auto_tx
                    .clone()
                    .unwrap()
                    .send(Duration::from_secs(1))
                    .await?;
                tx.send(()).await?;
            } else {
                //println!("准备停止");
                if window.get_is_auto() {
                    //println!("正在自动");
                    tx.send(()).await?;
                }
                window.set_is_auto(false);
                //println!("停止自动");
            }
        }

        Ok(())
    }

    pub async fn execute_skip(&mut self, tx: Sender<()>, source: bool) -> Result<(), EngineError> {
        if let Some(window) = self.weak.upgrade() {
            if source {
                //println!("发送");
                tx.send(()).await?;
            } else {
                //println!("准备停止");
                if window.get_is_skip() {
                    //println!("正在快进");
                    tx.send(()).await?;
                }
                window.set_is_skip(false);
                //println!("停止快进");
            }
        }

        Ok(())
    }

    pub async fn execute_script(&mut self) -> Result<(), EngineError> {
        println!("执行点击");
        {
            let scr = self.script.clone();
            let scr = scr.borrow();
            if scr.clear.get(&scr.index()).is_some() {
                DelayTX::clear_tx(&self.delay_tx).send(()).await?;
                DelayTX::clear_tx(&self.delay_move_tx).send(()).await?;
                DelayTX::clear_tx(&self.loop_move_tx).send(()).await?;
            } else {
                DelayTX::skip_tx(&self.delay_tx).send(()).await?;
                DelayTX::skip_tx(&self.delay_move_tx).send(()).await?;
                DelayTX::skip_tx(&self.loop_move_tx).send(()).await?;
            }
        }

        {
            let mut text = self.text.write().unwrap();
            if text.is_running {
                text.end();
                return Ok(());
            }
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
            Commands::OneCmd(command) => self.apply_command(command).await?,
            Commands::VarCmds(vars) => {
                let mut delay = Duration::default();
                for command in vars {
                    delay += self.apply_command(command).await?;
                }
                delay
            }
        };

        if is_wait {
            duration += delay;
        }

        if is_auto {
            //println!("script:{:?}", duration);
            self.auto_tx.clone().unwrap().send(duration).await?;
        }

        Ok(())
    }

    pub async fn apply_command(&mut self, command: Command) -> Result<Duration, EngineError> {
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
                self.show_bg(bg).await?;
            }
            if let Play(bgm) = pre_bgm {
                self.play_bgm(bgm).await?;
            } else if let PreBgm::Stop = pre_bgm {
                let bgm_player = self.bgm_player.borrow_mut();
                bgm_player.stop();
            }
            if let Some(figures) = pre_fg {
                for figure in figures.0.values() {
                    self.show_fg(&figure.clone()).await?;
                }
            }

            match command {
                Command::SetBackground(bg) => self.show_bg(bg).await?,
                Command::PlayBgm(bgm) => {
                    let mut script = self.script.borrow_mut();
                    if bgm != script.current_bgm() {
                        script.set_current_bgm(bgm.clone());
                        self.play_bgm(bgm).await?;
                    }
                }
                Command::Choice((explain, choices)) => {
                    *self.choose_lock.borrow_mut() = true;

                    let mut script = self.script.borrow_mut();
                    script.set_explain(&format!("选择支：{}", explain));
                    let mut choose_branch = Vec::with_capacity(choices.len());
                    for (index, choice) in choices.iter().enumerate() {
                        choose_branch.push((index as i32, SharedString::from(choice.0.clone())));
                    }
                    script.push_backlog("选择支".to_shared_string(), explain.to_shared_string());
                    window.set_choose_branch(Rc::new(VecModel::from(choose_branch)).into());
                    window.set_current_choose(choices.len() as i32);
                }
                Command::Dialogue { speaker, text } => {
                    let mut script = self.script.borrow_mut();
                    script.set_explain(&text);
                    script.push_backlog(speaker.to_shared_string(), text.to_shared_string());
                    window.set_speaker(SharedString::from(speaker));
                    {
                        let mut send_text = self.text.write().unwrap();
                        send_text.start_animation(text, window.get_text_speed());
                    }
                    let tx = self.text_tx.clone().unwrap();
                    tx.send(self.text.clone()).await?;
                }
                Command::PlayVoice {
                    ref name,
                    ref voice,
                } => {
                    if let Some(length) = VOICE_CONFIG.find(name) {
                        let voice_player = self.voice_player.borrow_mut();
                        let volume = window.get_main_volume() / 100.0;
                        let voice_volume = window.get_voice_volume() / 100.0;
                        voice_player.play_voice(
                            &format!("{}/{}/{}.ogg", ENGINE_CONFIG.voice_path(), name, voice),
                            volume * voice_volume,
                        );
                        duration += length.get(voice).unwrap().clone();
                    }
                }
                Command::Figure { .. } => {
                    self.show_fg(&command).await?;
                }
                Command::Move { .. } => {
                    self.show_move(&command).await?;
                }
                Command::Clear(distance, position) => self.clean_fg(&distance, &position).await?,
                Command::Jump(jump) => {
                    self.execute_jump(Jump::Label(jump)).await?;
                }
                Command::Label(_) => (),
                Command::Empty => (),
            }
        };

        //println!("apply cmd:{:?}", duration);
        Ok(duration)
    }

    async fn play_bgm(&self, bgm: String) -> Result<(), EngineError> {
        let weak = self.weak.clone();

        if let Some(window) = weak.upgrade() {
            let bgm_player = self.bgm_player.borrow_mut();
            let volume = window.get_main_volume() / 100.0;
            let bgm_volume = window.get_bgm_volume() / 100.0;
            bgm_player.play_loop(
                &format!("{}{}.ogg", ENGINE_CONFIG.bgm_path(), bgm),
                volume * bgm_volume,
            );
        }

        Ok(())
    }

    async fn show_bg(&self, bg: String) -> Result<(), EngineError> {
        let weak = self.weak.clone();

        if let Some(window) = weak.upgrade() {
            let image = Image::load_from_path(Path::new(&format!(
                "{}{}.png",
                ENGINE_CONFIG.background_path(),
                bg
            )))
            .unwrap();
            window.set_bg(image);
        }

        Ok(())
    }

    pub(crate) async fn show_fg(&self, fg: &Command) -> Result<(), EngineError> {
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
            if let Some(_) = delay {
                DelayTX::delay_tx(&self.delay_tx).send(fg.clone()).await?;
                return Ok(());
            }
            if let (Some(body_para), Some(face_para), Some(offset)) = FIGURE_CONFIG.find(&name) {
                let body_exist = match (&position[..], &distance[..]) {
                    ("-2", "z1") => window.get_fg_z1__2(),
                    ("0", "z1") => window.get_fg_z1_0(),
                    ("2", "z1") => window.get_fg_z1_2(),
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
                        ("-2", "z1") => window.set_fg_z1__2((body, *offset, *rate)),
                        ("0", "z1") => window.set_fg_z1_0((body, *offset, *rate)),
                        ("2", "z1") => window.set_fg_z1_2((body, *offset, *rate)),
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
                    ("-2", "z1") => window.set_face_z1__2((face, *face_x, *face_y)),
                    ("0", "z1") => window.set_face_z1_0((face, *face_x, *face_y)),
                    ("2", "z1") => window.set_face_z1_2((face, *face_x, *face_y)),
                    ("0", "no") => window.set_face_no_0((face, *face_x, *face_y)),
                    _ => unreachable!(),
                }
            }
        }

        Ok(())
    }

    pub(crate) async fn show_move(&self, fg_move: &Command) -> Result<(), EngineError> {
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
            if let Some(_) = delay {
                DelayTX::delay_tx(&self.delay_tx)
                    .send(fg_move.clone())
                    .await?;
                return Ok(());
            }

            let offset: (f32, f32) = match &action[..] {
                "to2" => {
                    if *repeat != 1 {
                        let tx = DelayTX::delay_tx(&self.loop_move_tx);
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
                        send_loop(tx.clone(), back);
                        send_loop(tx, action);
                    } else {
                        let tx = DelayTX::delay_tx(&self.delay_move_tx);
                        tx.send(Command::Figure {
                            name: name.to_string(),
                            distance: distance.to_string(),
                            body: body.to_string(),
                            face: face.to_string(),
                            position: "2".to_string(),
                            delay: Some("150".to_string()),
                        })
                            .await?;
                        tx.send(fg_move.back_and_clean())
                            .await?;
                    }
                    match (&position[..], &distance[..]) {
                        ("0", "z1") => (window.get_container_width() * 0.17, 0.0),
                        _ => unreachable!(),
                    }
                }
                "to0" => {
                    let tx = DelayTX::delay_tx(&self.delay_move_tx);
                    tx.send(Command::Figure {
                        name: name.to_string(),
                        distance: distance.to_string(),
                        body: body.to_string(),
                        face: face.to_string(),
                        position: "0".to_string(),
                        delay: Some("150".to_string()),
                    })
                    .await?;
                    tx.send(fg_move.back())
                    .await?;

                    match (&position[..], &distance[..]) {
                        ("2", "z1") => (-window.get_container_width() * 0.17, 0.0),
                        _ => unreachable!(),
                    }
                }
                "nod" => {
                    if *repeat != 1 {
                        //println!("发送循环，循环还剩{}次", repeat);
                        let tx = DelayTX::delay_tx(&self.loop_move_tx);
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
                        send_loop(tx.clone(), back);
                        send_loop(tx, nod);
                    } else {
                        let tx = DelayTX::delay_tx(&self.delay_move_tx);
                        tx.send(Command::Move {
                            name: name.to_string(),
                            distance: distance.to_string(),
                            body: body.to_string(),
                            face: face.to_string(),
                            position: position.to_string(),
                            action: "back".to_string(),
                            repeat: *repeat,
                            delay: Some("150".to_string()),
                        })
                            .await?;
                        //println!("点头");
                    }
                    (0.0, window.get_container_height() / 40.0)
                }
                "back" => {
                    //println!("归位");
                    (0.0, 0.0)
                },
                "back_and_clean" => {
                    match (&position[..], &distance[..]) {
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
                        _ => unreachable!(),
                    }
                    (0.0, 0.0)
                }
                _ => unreachable!(),
            };

            match (&position[..], &distance[..]) {
                ("-2", "z1") => window.set_offset_z1__2(offset),
                ("0", "z1") => window.set_offset_z1_0(offset),
                ("2", "z1") => window.set_offset_z1_2(offset),
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    async fn clean_fg(&self, distance: &str, position: &str) -> Result<(), EngineError> {
        let weak = self.weak.clone();
        if let Some(window) = weak.upgrade() {
            match (&position[..], &distance[..]) {
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
}

fn send_loop(tx: Sender<Command>, cmd: Command) {
    match tx.try_send(cmd) {
        Ok(_) => {}
        Err(tokio::sync::mpsc::error::TrySendError::Full(cmd)) => {
            // 通道满了：把发送任务交给 tokio 等待
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                if let Err(e) = tx_clone.send(cmd).await {
                    eprintln!("delay tx send failed: {:?}", e);
                }
            });
        }
        Err(e) => {
            eprintln!("try_send other error: {:?}", e);
        }
    }
}

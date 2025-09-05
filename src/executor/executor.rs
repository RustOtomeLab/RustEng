use crate::audio::player::PreBgm::Play;
use crate::audio::player::{Player, PreBgm};
use crate::config::figure::FIGURE_CONFIG;
use crate::config::save_load::SaveData;
use crate::config::user::save_user_config;
use crate::config::voice::VOICE_CONFIG;
use crate::config::ENGINE_CONFIG;
use crate::error::EngineError;
use crate::parser::parser::{Command, Commands};
use crate::script::{Label, Script};
use crate::ui::ui::MainWindow;
use slint::{Image, Model, SharedString, ToSharedString, VecModel, Weak};
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;

pub(crate) enum Jump {
    Label(Label),
    Index((String, i32)),
}

fn figure_default() -> (Image, Image, f32, f32, f32) {
    (Image::default(), Image::default(), 0.0, 0.0, 0.0)
}

pub struct Executor {
    script: Rc<RefCell<Script>>,
    bgm_player: Rc<RefCell<Player>>,
    voice_player: Rc<RefCell<Player>>,
    weak: Weak<MainWindow>,
    choose_lock: Rc<RefCell<bool>>,
    delay_tx: Option<Sender<Command>>,
    auto_tx: Option<Sender<Duration>>,
    fg_skip_tx: Option<Sender<()>>,
}

impl Clone for Executor {
    fn clone(&self) -> Executor {
        Executor {
            script: self.script.clone(),
            bgm_player: self.bgm_player.clone(),
            voice_player: self.voice_player.clone(),
            weak: self.weak.clone(),
            choose_lock: self.choose_lock.clone(),
            delay_tx: self.delay_tx.clone(),
            auto_tx: self.auto_tx.clone(),
            fg_skip_tx: self.fg_skip_tx.clone(),
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
            choose_lock: Rc::new(RefCell::new(false)),
            delay_tx: None,
            auto_tx: None,
            fg_skip_tx: None,
        }
    }

    pub fn get_weak(&self) -> Weak<MainWindow> {
        self.weak.clone()
    }

    pub fn set_delay_tx(&mut self, delay_tx: Sender<Command>) {
        self.delay_tx = Some(delay_tx);
    }

    pub fn set_auto_tx(&mut self, auto_tx: Sender<Duration>) {
        self.auto_tx = Some(auto_tx);
    }

    pub fn set_fg_skip_tx(&mut self, fg_skip_tx: Sender<()>) {
        self.fg_skip_tx = Some(fg_skip_tx);
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
                let mut content = String::default();
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
                content = toml::to_string_pretty(&sava_data)?;
                fs::write(format!("{}{}.toml", ENGINE_CONFIG.save_path(), i), content)?;
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
        self.fg_skip_tx.clone().unwrap().send(()).await?;

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
                for figure in figures {
                    self.show_fg(&figure).await?;
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
                    window.set_dialogue(SharedString::from(text));
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

    async fn show_fg(&self, fg: &Command) -> Result<Duration, EngineError> {
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
                let tx = self.delay_tx.clone().unwrap();
                tx.send(fg.clone()).await?;
                return Ok(Duration::from_secs(0));
            }
            if let (Some(body_para), Some(face_para)) = FIGURE_CONFIG.find(&name) {
                let mut rate = body_para.get(body).unwrap_or(&0.0);
                let (face_x, face_y) = face_para.get(face).unwrap();
                let body = if !body.is_empty() {
                    Image::load_from_path(Path::new(&format!(
                        "{}{}/{}/{}.png",
                        ENGINE_CONFIG.figure_path(),
                        name,
                        distance,
                        body
                    )))
                    .unwrap()
                } else {
                    let script = self.script.borrow();
                    let (body, _) = script.find_latest_fg(&script.index(), &distance, &position);
                    rate = body_para.get(&body).unwrap();
                    Image::load_from_path(Path::new(&format!(
                        "{}{}/{}/{}.png",
                        ENGINE_CONFIG.figure_path(),
                        name,
                        distance,
                        body
                    )))
                    .unwrap()
                };
                let face = Image::load_from_path(Path::new(&format!(
                    "{}{}/{}/{}.png",
                    ENGINE_CONFIG.figure_path(),
                    name,
                    distance,
                    face
                )))
                .unwrap();
                match (&position[..], &distance[..]) {
                    ("-2", "z1") => window.set_fg_z1__2((body, face, *face_x, *face_y, *rate)),
                    ("0", "z1") => window.set_fg_z1_0((body, face, *face_x, *face_y, *rate)),
                    ("2", "z1") => window.set_fg_z1_2((body, face, *face_x, *face_y, *rate)),
                    ("0", "no") => window.set_fg_no_0((body, face, *face_x, *face_y, *rate)),
                    _ => unreachable!(),
                }
            }
        }

        Ok(Duration::from_secs(0))
    }

    async fn clean_fg(&self, distance: &str, position: &str) -> Result<(), EngineError> {
        let weak = self.weak.clone();
        if let Some(window) = weak.upgrade() {
            match (&position[..], &distance[..]) {
                ("-2", "z1") => window.set_fg_z1__2(figure_default()),
                ("2", "z1") => window.set_fg_z1_2(figure_default()),
                ("0", "z1") => window.set_fg_z1_0(figure_default()),
                ("0", "no") => window.set_fg_no_0(figure_default()),
                ("All", "All") => {
                    window.set_fg_z1__2(figure_default());
                    window.set_fg_z1_0(figure_default());
                    window.set_fg_z1_2(figure_default());
                    window.set_fg_no_0(figure_default());
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }
}

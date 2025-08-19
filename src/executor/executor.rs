use crate::audio::player::PreBgm::Play;
use crate::audio::player::{Player, PreBgm};
use crate::error::EngineError;
use crate::parser::parser::{Command, Commands};
use crate::script::{Label, Script};
use crate::ui::ui::MainWindow;
use slint::{Image, Model, SharedString, ToSharedString, VecModel, Weak};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use crate::config::ENGINE_CONFIG;
// static BACKGROUND_PATH: &str = "./source/background/";
// static VOICE_PATH: &str = "./source/voice/";
// static BGM_PATH: &str = "./source/bgm/";
// static FIGURE_PATH: &str = "./source/figure/";

pub(crate) enum Jump {
    Label(Label),
    Index((String, i32)),
}

pub struct Executor {
    script: Rc<RefCell<Script>>,
    bgm_player: Rc<RefCell<Player>>,
    voice_player: Rc<RefCell<Player>>,
    weak: Weak<MainWindow>,
    choose_lock: Rc<RefCell<bool>>,
}

impl Clone for Executor {
    fn clone(&self) -> Executor {
        Executor {
            script: self.script.clone(),
            bgm_player: self.bgm_player.clone(),
            voice_player: self.voice_player.clone(),
            weak: self.weak.clone(),
            choose_lock: self.choose_lock.clone(),
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
        }
    }

    pub fn get_weak(&self) -> Weak<MainWindow> {
        self.weak.clone()
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
            let mut save_items = Vec::with_capacity(index as usize);
            let exists_save_items = window.get_save_items();
            for (i, item) in exists_save_items.iter().enumerate() {
                if i != index as usize {
                    save_items.push(item);
                } else {
                    save_items.push((
                        bg.clone(),
                        SharedString::from(script.explain()),
                        script.index() as i32,
                        SharedString::from(script.name()),
                    ))
                }
            }
            //println!("{:#?}", save_items);
            window.set_save_items(Rc::new(VecModel::from(save_items)).into());

            //println!("save {}", index);
        }

        Ok(())
    }

    pub async fn execute_load(&mut self, name: String, index: i32) -> Result<(), EngineError> {
        let mut volume = 0.0;
        if !name.is_empty() {
            let weak = self.weak.clone();
            if let Some(window) = weak.upgrade() {
                volume = window.get_main_volume() * window.get_bgm_volume() / 10000.0;
                window.set_current_screen(2);
                window.set_current_choose(0);
            }
            self.execute_jump(volume, Jump::Index((name, index - 1)))
                .await?;
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

    pub async fn execute_choose(&mut self, choice: SharedString) -> Result<(), EngineError> {
        *self.choose_lock.borrow_mut() = false;

        let label: (String, String);
        {
            let scr = self.script.clone();
            let scr = scr.borrow();
            label = scr.get_choice_label(&choice).unwrap().clone();
        }

        let mut volume = 0.0;
        if let Some(window) = self.weak.upgrade() {
            window.set_choose_branch(Rc::new(VecModel::from(vec![])).into());
            window.set_current_choose(0);
            window.set_speaker("".into());
            window.set_dialogue(choice);
            volume = window.get_main_volume() * window.get_bgm_volume() / 10000.0;
        }

        self.execute_jump(volume, Jump::Label(label)).await
    }

    pub async fn execute_jump(&mut self, volume: f32, label: Jump) -> Result<(), EngineError> {
        let mut script = self.script.borrow_mut();
        let current_bgm = script.current_bgm().to_string();
        let mut pre_bg = None;
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
                pre_bg = script.get_background(index).map(|(i, bg)| bg.clone());
            }
        }
        script.set_pre_bg(pre_bg);
        script.set_pre_bgm(pre_bgm);
        script.set_index(current_block);

        Ok(())
    }

    pub async fn execute_script(&mut self) -> Result<(), EngineError> {
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
        match commands {
            Commands::EmptyCmd => unreachable!(),
            Commands::OneCmd(command) => {
                self.apply_command(command).await?;
            }
            Commands::VarCmds(vars) => {
                for command in vars {
                    self.apply_command(command).await?;
                }
            }
        }
        Ok(())
    }

    async fn apply_command(&mut self, command: Command) -> Result<(), EngineError> {
        if let Some(window) = self.weak.upgrade() {
            let pre_bg;
            let pre_bgm;
            {
                let mut scr = self.script.borrow_mut();
                pre_bg = scr.pre_bg();
                pre_bgm = scr.pre_bgm();
            }
            if let Some(bg) = pre_bg {
                let image =
                    Image::load_from_path(Path::new(&format!("{}{}.png", ENGINE_CONFIG.background_path(), bg)))
                        .unwrap();
                window.set_bg(image);
            }
            if let Play(bgm) = pre_bgm {
                let bgm_player = self.bgm_player.borrow_mut();
                let volume = window.get_main_volume() / 100.0;
                let bgm_volume = window.get_bgm_volume() / 100.0;
                bgm_player.play_loop(&format!("{}{}.ogg", ENGINE_CONFIG.bgm_path(), bgm), volume * bgm_volume);
            } else if let PreBgm::Stop = pre_bgm {
                let bgm_player = self.bgm_player.borrow_mut();
                bgm_player.stop();
            }

            match command {
                Command::SetBackground(bg) => {
                    let image =
                        Image::load_from_path(Path::new(&format!("{}{}.png", ENGINE_CONFIG.background_path(), bg)))
                            .unwrap();
                    window.set_bg(image);
                }
                Command::PlayBgm(bgm) => {
                    let mut script = self.script.borrow_mut();
                    if bgm != script.current_bgm() {
                        script.set_current_bgm(bgm.clone());
                        let bgm_player = self.bgm_player.borrow_mut();
                        let volume = window.get_main_volume() / 100.0;
                        let bgm_volume = window.get_bgm_volume() / 100.0;
                        bgm_player
                            .play_loop(&format!("{}{}.ogg", ENGINE_CONFIG.bgm_path(), bgm), volume * bgm_volume);
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
                Command::PlayVoice(voice) => {
                    let voice_player = self.voice_player.borrow_mut();
                    let volume = window.get_main_volume() / 100.0;
                    let voice_volume = window.get_voice_volume() / 100.0;
                    voice_player.play_voice(
                        &format!("{}{}.ogg", ENGINE_CONFIG.voice_path(), voice),
                        volume * voice_volume,
                    );
                }
                Command::Figure {
                    name,
                    distance,
                    body,
                    face,
                    position,
                } => {
                    let body = Image::load_from_path(Path::new(&format!(
                        "{}{}/{}/{}.png",
                        ENGINE_CONFIG.figure_path(), name, distance, body
                    )))
                    .unwrap();
                    let face = Image::load_from_path(Path::new(&format!(
                        "{}{}/{}/{}.png",
                        ENGINE_CONFIG.figure_path(), name, distance, face
                    )))
                    .unwrap();
                    match &position[..] {
                        "0" => {
                            window.set_fg_body_0(body);
                            window.set_fg_face_0(face);
                        }
                        _ => (),
                    }
                }
                Command::Jump(jump) => {
                    let volume = window.get_main_volume() * window.get_bgm_volume() / 10000.0;
                    self.execute_jump(volume, Jump::Label(jump)).await?;
                }
                Command::Label(label) => (),
            }
        };
        Ok(())
    }
}

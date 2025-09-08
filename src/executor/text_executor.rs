use std::sync::mpsc::Receiver;
use crate::executor::executor::Executor;
use crate::parser::parser::Command;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::RwLock;
use slint::{ToSharedString, Weak};
use tokio::sync::mpsc::{Sender, channel};
use tokio::time::{sleep, Duration};
use crate::ui::ui::MainWindow;

pub struct TextExecutor {
    timer: slint::Timer,
    weak: Weak<MainWindow>,
    text_rx: Option<Receiver<String>>,
}

impl TextExecutor {
    pub fn new(executor: Executor) -> (Self, Sender<Arc<RwLock<DisplayText>>>) {
        let (full_text_tx, mut full_text_rx) = channel::<Arc<RwLock<DisplayText>>>(10);
        let (text_tx, text_rx) = std::sync::mpsc::channel::<String>();
        let timer = slint::Timer::default();
        let weak = executor.get_weak();

        let executor = Self {
            timer,
            weak,
            text_rx: Some(text_rx),
        };

        tokio::spawn(async move {
            while let Some(text) = full_text_rx.recv().await {
                while text.read().unwrap().is_running {
                    let tx = text_tx.clone();
                    {
                        let mut text = text.write().unwrap();
                        if let Some(text) = text.next_character() {
                            tx.send(text).unwrap()
                        }
                    }
                    sleep(Duration::from_millis(30)).await;
                }
            }
        });

        (executor, full_text_tx)
    }

    pub fn start_timer(&mut self) {
        //println!("定时器打开");
        let weak = self.weak.clone();
        let mut rx = self.text_rx.take().unwrap();

        self.timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(30),
            move || {
                if let Ok(text) = rx.try_recv() {
                    if let Some(window) = weak.upgrade() {
                        window.set_dialogue(text.to_shared_string());
                    }
                }
            },
        );
    }
}


pub struct DisplayText {
    pub(crate) full_text: String,
    current_index: usize,
    pub(crate) is_running: bool,
}

impl DisplayText {
    pub fn new() -> Self {
        Self {
            full_text: String::new(),
            current_index: 0,
            is_running: false,
        }
    }

    pub fn start_animation(&mut self, text: String) {
        self.full_text = text;
        self.current_index = 0;
        self.is_running = true;
    }

    pub fn end(&mut self) {
        self.current_index = self.full_text.chars().count() - 1;
    }

    fn next_character(&mut self) -> Option<String> {
        if !self.is_running {
            return None;
        }

        if self.current_index >= self.full_text.chars().count() {
            self.is_running = false;
            return None;
        }

        let chars: Vec<char> = self.full_text.chars().collect();
        let displayed_text: String = chars[..=self.current_index].iter().collect();
        self.current_index += 1;

        Some(displayed_text)
    }
}
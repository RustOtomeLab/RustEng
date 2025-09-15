use crate::executor::executor::Executor;
use crate::ui::ui::MainWindow;
use slint::{ToSharedString, Weak};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::sync::mpsc::{channel, Sender};
use tokio::time::{sleep, Duration};

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
                let speed = text.read().unwrap().speed;
                if speed == Duration::from_millis(0) {
                    let mut text = text.write().unwrap();
                    let tx = text_tx.clone();
                    tx.send(text.full_text.clone()).unwrap();
                    text.is_running = false;
                    continue;
                }
                while text.read().unwrap().is_running {
                    let tx = text_tx.clone();
                    {
                        let mut text = text.write().unwrap();
                        if let Some(text) = text.next_character() {
                            tx.send(text).unwrap()
                        }
                    }
                    sleep(speed).await;
                }
            }
        });

        (executor, full_text_tx)
    }

    pub fn start_timer(&mut self) {
        //println!("定时器打开");
        let weak = self.weak.clone();
        let rx = self.text_rx.take().unwrap();

        self.timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(20),
            move || {
                if let Ok(text) = rx.try_recv() {
                    let mut parts = text.split("{nns}").map(str::trim);
                    let (t1, t2, t3) = (
                        parts.next().unwrap_or(""),
                        parts.next().unwrap_or(""),
                        parts.next().unwrap_or(""),
                    );
                    if let Some(window) = weak.upgrade() {
                        window.set_dialogue_1(t1.to_shared_string());
                        window.set_dialogue_2(t2.to_shared_string());
                        window.set_dialogue_3(t3.to_shared_string());
                    }
                }
            },
        );
    }
}

pub struct DisplayText {
    pub(crate) full_text: String,
    pub(crate) speed: Duration,
    current_index: usize,
    pub(crate) is_running: bool,
}

impl DisplayText {
    pub fn new() -> Self {
        Self {
            full_text: String::new(),
            speed: Duration::default(),
            current_index: 0,
            is_running: false,
        }
    }

    pub fn start_animation(&mut self, text: String, speed: f32) {
        self.full_text = text;
        self.speed = Duration::from_millis(speed as u64);
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
        if self.current_index + 5 <= self.full_text.chars().count()
            && chars[self.current_index..self.current_index + 5] == ['{', 'n', 'n', 's', '}']
        {
            self.current_index += 5;
        }
        let displayed_text: String = chars[..=self.current_index].iter().collect();
        self.current_index += 1;

        Some(displayed_text)
    }
}

use std::time::Duration;
use crate::executor::executor::Executor;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::thread::sleep;
use tokio::sync::mpsc::Sender;

pub struct AutoExecutor {
    timer: slint::Timer,
    pub executor: Executor,
    is_auto: Arc<AtomicBool>,
    auto_rx: Option<Receiver<bool>>,
}

impl AutoExecutor {
    pub fn new(executor: Executor) -> (Self, Sender<bool>, Sender<Duration>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<bool>(10);
        let (delay_tx, mut delay_rx) = tokio::sync::mpsc::channel::<Duration>(10);
        let (auto_tx, auto_rx) = std::sync::mpsc::channel::<bool>();
        let is_auto = Arc::new(AtomicBool::new(false));

        // 创建定时器
        let timer = slint::Timer::default();

        let executor = Self {
            timer,
            executor,
            is_auto: is_auto.clone(),
            auto_rx: Some(auto_rx),
        };

        // 监听控制信号的任务
        let is_auto_clone = is_auto.clone();
        tokio::spawn(async move {
            let mut start = true;
            while let Some(_) = rx.recv().await {
                if start {
                    println!("开始自动");
                    is_auto_clone.store(true, Ordering::Relaxed);
                    start = false;
                } else {
                    println!("停止自动");
                    is_auto_clone.store(false, Ordering::Relaxed);
                    start = true;
                }
            }
        });

        tokio::spawn(async move {
            while let Some(delay) = delay_rx.recv().await {
                sleep(delay);
                auto_tx.send(true).unwrap();
            }
        });

        (executor, tx, delay_tx)
    }

    pub fn start_timer(&mut self) {
        println!("定时器打开");
        let executor = self.executor.clone();
        let is_auto = self.is_auto.clone();
        let rx = self.auto_rx.take().unwrap();

        self.timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(100),
            move || {
                if is_auto.load(Ordering::Relaxed) {
                    if let Ok(rx) = rx.recv() {
                        println!("定时器触发 - 自动执行");

                        let mut executor = executor.clone();
                        slint::spawn_local(async move { executor.execute_script().await })
                            .expect("Clicked panicked");
                    }
                }
            },
        );
    }
}

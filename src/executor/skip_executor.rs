use crate::executor::executor::Executor;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{channel, Sender};

pub struct SkipExecutor {
    timer: slint::Timer,
    pub executor: Executor,
    is_skip: Arc<AtomicBool>,
}

impl SkipExecutor {
    pub fn new(executor: Executor) -> (Self, Sender<()>) {
        let (tx, mut rx) = channel::<()>(10);
        let is_skip = Arc::new(AtomicBool::new(false));

        // 创建定时器
        let timer = slint::Timer::default();

        let executor = Self {
            timer,
            executor,
            is_skip: is_skip.clone(),
        };

        // 监听控制信号的任务
        let is_skip_clone = is_skip.clone();
        tokio::spawn(async move {
            let mut start = true;
            while let Some(_) = rx.recv().await {
                if start {
                    println!("开始快进");
                    is_skip_clone.store(true, Ordering::Relaxed);
                    start = false;
                } else {
                    println!("停止快进");
                    is_skip_clone.store(false, Ordering::Relaxed);
                    start = true;
                }
            }
        });

        (executor, tx)
    }

    pub fn start_timer(&mut self) {
        let executor = self.executor.clone();
        let is_skip = self.is_skip.clone();

        self.timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(100),
            move || {
                if is_skip.load(Ordering::Relaxed) {
                    println!("定时器触发 - 快速执行");

                    let mut executor = executor.clone();
                    slint::spawn_local(async move { executor.execute_script().await })
                        .expect("Clicked panicked");
                }
            },
        );
    }
}

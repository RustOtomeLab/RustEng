use crate::executors::executor::Executor;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::mpsc::{channel, Sender};

pub(crate) struct SkipExecutor {
    timer: slint::Timer,
    pub(crate) executor: Executor,
    is_skip: Arc<AtomicBool>,
}

impl SkipExecutor {
    pub(crate) fn new(executor: Executor) -> (Self, Sender<()>) {
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
            while (rx.recv().await).is_some() {
                if start {
                    is_skip_clone.store(true, Ordering::Relaxed);
                    start = false;
                } else {
                    is_skip_clone.store(false, Ordering::Relaxed);
                    start = true;
                }
            }
        });

        (executor, tx)
    }

    pub(crate) fn start_timer(&mut self) {
        let executor = self.executor.clone();
        let is_skip = self.is_skip.clone();

        self.timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(100),
            move || {
                if is_skip.load(Ordering::Relaxed) {
                    let mut executor = executor.clone();
                    slint::spawn_local(async move {
                        if let Err(e) = executor.execute_script() {
                            eprintln!("skip execute_script failed: {e}");
                        }
                    })
                    .expect("skip-play timer: no slint event loop");
                }
            },
        );
    }
}

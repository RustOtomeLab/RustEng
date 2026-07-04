use crate::executors::executor::Executor;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::Receiver,
    Arc,
};
use tokio::{
    sync::mpsc::{channel, Sender},
    time::{Duration, Sleep},
};

pub struct AutoExecutor {
    timer: slint::Timer,
    pub executor: Executor,
    is_auto: Arc<AtomicBool>,
    auto_rx: Option<Receiver<()>>,
}

impl AutoExecutor {
    pub fn new(executor: Executor) -> (Self, Sender<()>, Sender<Duration>) {
        let (tx, mut rx) = channel::<()>(10);
        let (auto_delay_tx, mut auto_delay_rx) = channel::<Duration>(10);
        let (auto_tx, auto_rx) = std::sync::mpsc::channel::<()>();
        let is_auto = Arc::new(AtomicBool::new(false));

        let (reset_tx, mut reset_rx) = channel::<()>(10);

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
            while rx.recv().await.is_some() {
                if start {
                    is_auto_clone.store(true, Ordering::Relaxed);
                    start = false;
                } else {
                    is_auto_clone.store(false, Ordering::Relaxed);
                    start = true;
                    if let Err(e) = reset_tx.send(()).await {
                        eprintln!("auto reset channel closed: {e}");
                        return;
                    }
                }
            }
        });

        tokio::spawn(async move {
            let mut current_delay: Option<Sleep> = None;

            loop {
                tokio::select! {
                    Some(delay) = auto_delay_rx.recv() => {
                        current_delay = Some(tokio::time::sleep(delay));
                    }

                    // 延迟完成
                    _ = async {
                        if let Some(sleep) = current_delay {
                            sleep.await
                        } else {
                            std::future::pending::<()>().await
                        }
                    } => {
                        if let Err(e) = auto_tx.send(()) {
                            eprintln!("auto trigger channel closed: {e}");
                            return;
                        }
                        current_delay = None;
                    }

                    // 重置请求
                    _ = reset_rx.recv() => {
                        current_delay = None;
                    }
                }
            }
        });

        (executor, tx, auto_delay_tx)
    }

    pub fn start_timer(&mut self) {
        let executor = self.executor.clone();
        let is_auto = self.is_auto.clone();
        let rx = self.auto_rx.take().unwrap();

        self.timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(100),
            move || {
                if is_auto.load(Ordering::Relaxed) && rx.try_recv().is_ok() {
                    let mut executor = executor.clone();
                    slint::spawn_local(async move {
                        if let Err(e) = executor.execute_script().await {
                            eprintln!("auto execute_script failed: {e}");
                        }
                    })
                    .expect("auto-play timer: no slint event loop");
                }
            },
        );
    }
}

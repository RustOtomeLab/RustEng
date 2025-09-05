use crate::executor::executor::Executor;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Sender};
use tokio::time::{Duration, Sleep};

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
            while let Some(_) = rx.recv().await {
                if start {
                    //println!("开始自动");
                    is_auto_clone.store(true, Ordering::Relaxed);
                    start = false;
                } else {
                    //println!("停止自动");
                    is_auto_clone.store(false, Ordering::Relaxed);
                    start = true;
                    reset_tx.send(()).await.unwrap();
                }
            }
        });

        tokio::spawn(async move {
            let mut current_delay: Option<Sleep> = None;

            loop {
                tokio::select! {
                    Some(delay) = auto_delay_rx.recv() => {
                        //println!("设置新的延迟: {:?}", delay);
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
                        //println!("准备自动");
                        auto_tx.send(()).unwrap();
                        current_delay = None;
                    }

                    // 重置请求
                    _ = reset_rx.recv() => {
                        //println!("reset");
                        current_delay = None;
                    }
                }
            }
        });

        (executor, tx, auto_delay_tx)
    }

    pub fn start_timer(&mut self) {
        //println!("定时器打开");
        let executor = self.executor.clone();
        let is_auto = self.is_auto.clone();
        let rx = self.auto_rx.take().unwrap();

        self.timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(100),
            move || {
                if is_auto.load(Ordering::Relaxed) {
                    if let Ok(_) = rx.try_recv() {
                        //println!("定时器触发 - 自动执行");

                        let mut executor = executor.clone();
                        slint::spawn_local(async move { executor.execute_script().await })
                            .expect("Clicked panicked");
                    }
                }
            },
        );
    }
}

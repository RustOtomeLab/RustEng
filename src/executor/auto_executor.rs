use crate::executor::executor::Executor;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub struct AutoExecutor {
    timer: slint::Timer,
    executor: Executor,
    is_auto: Arc<AtomicBool>,
}

impl AutoExecutor {
    pub fn new(executor: Executor) -> (Self, Sender<bool>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<bool>(10);
        let is_auto = Arc::new(AtomicBool::new(false));

        // 创建定时器
        let timer = slint::Timer::default();

        let executor = Self {
            timer,
            executor,
            is_auto: is_auto.clone(),
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

        (executor, tx)
    }

    pub fn start_timer(&self) {
        println!("定时器打开");
        let executor = self.executor.clone();
        let is_auto = self.is_auto.clone();

        self.timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_secs(5),
            move || {
                if is_auto.load(Ordering::Relaxed) {
                    println!("定时器触发 - 自动执行");

                    let mut executor = executor.clone();
                    slint::spawn_local(async move { executor.execute_script().await })
                        .expect("Clicked panicked");
                } else {
                    println!("定时器触发 - 自动已关闭");
                }
            },
        );
    }
}

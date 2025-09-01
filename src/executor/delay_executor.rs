use crate::executor::executor::Executor;
use crate::parser::parser::Command;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc::Sender;

pub struct DelayExecutor {
    timer: slint::Timer,
    executor: Executor,
    command: Arc<RwLock<Command>>,
}

impl DelayExecutor {
    pub fn new(executor: Executor) -> (Self, Sender<Command>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Command>(10);

        let timer = slint::Timer::default();
        let command = Arc::new(RwLock::new(Command::Empty));
        let command_clone = command.clone();

        let executor = Self {
            timer,
            executor,
            command,
        };
        tokio::spawn(async move {
            while let Some(command) = rx.recv().await {
                if let Command::Figure { delay, .. } = &command {
                    println!("收到delay指令");
                    sleep(Duration::from_millis(
                        delay.clone().unwrap().parse::<u64>().unwrap_or(0),
                    )).await;
                    println!("delay结束");
                    let mut cmd = command_clone.write().unwrap();
                    *cmd = command;
                }
            }
        });

        (executor, tx)
    }

    pub fn start_timer(&self) {
        let executor = self.executor.clone();
        let command = self.command.clone();

        self.timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(100),
            move || {
                let mut cmd = command.write().unwrap();
                if let Command::Figure {
                    name,
                    distance,
                    face,
                    body,
                    position,
                    ..
                } = cmd.clone()
                {
                    println!("准备执行");
                    let mut executor = executor.clone();
                    slint::spawn_local(async move {
                        executor
                            .apply_command(Command::Figure {
                                name,
                                distance,
                                face,
                                body,
                                position,
                                delay: None,
                            })
                            .await
                    })
                    .expect("Delay panicked");
                    *cmd = Command::Empty;
                }
            },
        );
    }
}

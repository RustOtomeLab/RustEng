use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread::sleep;
use crate::executor::executor::Executor;
use crate::parser::parser::Command;
use tokio::sync::mpsc::Sender;

pub struct DelayExecutor {
    timer: slint::Timer,
    executor: Executor,
    command: Arc<Option<Command>>,
}

impl DelayExecutor {
    pub fn new(executor: Executor) -> (Self, Sender<Command>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Command>(10);

        let timer = slint::Timer::default();
        let command = Arc::new(None);

        let executor = Self {
            timer,
            executor,
            command: command.clone(),
        };

        let mut command_clone = command.clone();
        tokio::spawn(async move {
            while let Some(command) = rx.recv().await {
                if let Command::Figure {delay,..} = &command {
                    println!("收到delay指令");
                    sleep(std::time::Duration::from_millis(delay.clone().unwrap().parse::<u64>().unwrap_or(0)));
                    println!("delay结束");
                }
                command_clone = Arc::new(Some(command));
            }
        });

        (executor, tx)
    }

    pub fn start_timer(&self) {
        let executor = self.executor.clone();
        let mut command = self.command.clone();

        self.timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_millis(100),
            move || {

                if let Some(command) = &*command {
                    println!("准备执行");
                    let u = command.clone();
                    let mut executor = executor.clone();
                    slint::spawn_local(async move { executor.apply_command(u).await })
                        .expect("Delay panicked");
                }
                command = Arc::new(None);
            },
        );
    }
}

use crate::executor::executor::Executor;
use crate::parser::parser::Command;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration};

#[derive(Clone)]
pub struct DelayTX {
    delay_tx: Sender<Command>,
    skip_tx: Sender<()>,
    clear_tx: Sender<()>,
}

impl DelayTX {
    pub fn delay_tx(tx: &Option<DelayTX>) -> Sender<Command> {
        if let Some(tx) = tx {
            tx.delay_tx.clone()
        } else {
            unreachable!()
        }
    }

    pub fn skip_tx(tx: &Option<DelayTX>) -> Sender<()> {
        if let Some(tx) = tx {
            tx.skip_tx.clone()
        } else {
            unreachable!()
        }
    }

    pub fn clear_tx(tx: &Option<DelayTX>) -> Sender<()> {
        if let Some(tx) = tx {
            tx.clear_tx.clone()
        } else {
            unreachable!()
        }
    }
}

pub struct DelayExecutor {
    timer: slint::Timer,
    pub(crate) executor: Executor,
    command: Arc<RwLock<VecDeque<Command>>>,
}

impl DelayExecutor {
    pub fn new(executor: Executor) -> (Self, DelayTX) {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Command>(10);
        let (skip_tx, mut skip_rx) = tokio::sync::mpsc::channel::<()>(10);
        let (clear_tx, mut clear_rx) = tokio::sync::mpsc::channel::<()>(10);

        let timer = slint::Timer::default();
        let command = Arc::new(RwLock::new(VecDeque::new()));
        let command_clone = command.clone();

        let executor = Self {
            timer,
            executor,
            command,
        };
        tokio::spawn(async move {
            let mut current_figure: VecDeque<Command> = VecDeque::new();

            loop {
                tokio::select! {
                    Some(figure) = rx.recv()=> {
                        //println!("收到delay指令");
                        current_figure.push_back(figure);
                    }

                    // 延迟完成
                    _ = async {
                        if let Some(Command::Figure {delay, ..})
                        | Some(Command::Move {delay, ..}) = current_figure.front(){
                            sleep(Duration::from_millis(
                                delay.clone().unwrap().parse::<u64>().unwrap_or(0),
                            )).await;
                        } else {
                            std::future::pending::<()>().await
                        }
                    } => {
                        //println!("delay结束");
                        command_clone.write().unwrap().push_back(current_figure.pop_front().unwrap());
                    }

                    // 重置请求
                    _ = skip_rx.recv() => {
                        //println!("立刻完成延时立绘");
                        while let Some(figure) = current_figure.pop_front() {
                            if let Command::Figure {..} = figure {
                                command_clone.write().unwrap().push_back(figure);
                            } else if let Command::Move {..} = figure {
                                if figure.action() != "nod" {
                                    command_clone.write().unwrap().push_back(figure)
                                }
                            }
                        }
                    }

                    // 清空请求
                    _ = clear_rx.recv() => {
                        current_figure.clear();
                    }
                }
            }
        });

        (
            executor,
            DelayTX {
                delay_tx: tx,
                skip_tx,
                clear_tx,
            },
        )
    }

    pub fn start_timer(&self) {
        let executor = self.executor.clone();
        let command = self.command.clone();

        self.timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(30),
            move || {
                if let Some(mut cmd) = command.write().unwrap().pop_front() {
                    //println!("准备执行");
                    cmd.delete_delay();
                    let executor = executor.clone();
                    slint::spawn_local(async move {
                        if let Command::Figure { .. } = &cmd {
                            executor.show_fg(&cmd).await.unwrap();
                        } else if let Command::Move { .. } = &cmd {
                            executor.show_move(&cmd).await.unwrap();
                        }
                    })
                    .expect("Delay panicked");
                }
            },
        );
    }
}

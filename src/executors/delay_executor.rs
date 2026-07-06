use crate::error::EngineError;
use crate::executors::executor::Executor;
use crate::parser::script_parser::Command;
use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};
use tokio::{
    sync::mpsc::Sender,
    time::{sleep, Duration},
};

#[derive(Clone)]
pub(crate) struct DelayTX {
    delay_tx: Sender<Command>,
    skip_tx: Sender<()>,
    clear_tx: Sender<()>,
}

#[derive(Clone)]
pub(crate) struct DelayChannels {
    pub(crate) delay_tx: DelayTX,
    pub(crate) delay_move_tx: DelayTX,
    pub(crate) loop_move_tx: DelayTX,
}

impl DelayChannels {
    pub(crate) fn send_delay(&self, fg: &Command) -> Result<(), EngineError> {
        self.delay_tx.delay_tx.try_send(fg.clone())?;
        Ok(())
    }

    pub(crate) fn send_move(&self, fg_move: Command) -> Result<(), EngineError> {
        self.delay_move_tx.delay_tx.try_send(fg_move)?;
        Ok(())
    }

    pub(crate) fn send_loop(&self, fg_move: Command, fg_back: Command) {
        try_send_loop(&self.delay_move_tx.delay_tx, fg_back);
        try_send_loop(&self.loop_move_tx.delay_tx, fg_move);
    }

    pub(crate) fn clear_all(&self) {
        self.delay_tx
            .clear_tx
            .try_send(())
            .expect("clear_delay_tx send fali");
        self.delay_move_tx
            .clear_tx
            .try_send(())
            .expect("clear_delay_move_tx send fali");
        self.loop_move_tx
            .clear_tx
            .try_send(())
            .expect("clear_loop_move_tx send fali");
    }

    pub(crate) fn skip_all(&self) {
        self.delay_tx
            .skip_tx
            .try_send(())
            .expect("skip_delay_tx send fali");
        self.delay_move_tx
            .skip_tx
            .try_send(())
            .expect("skip_delay_move_tx send fali");
        self.loop_move_tx
            .skip_tx
            .try_send(())
            .expect("skip_loop_move_tx send fali");
    }
}

fn try_send_loop(tx: &Sender<Command>, cmd: Command) {
    match tx.try_send(cmd) {
        Ok(_) => {}
        Err(tokio::sync::mpsc::error::TrySendError::Full(cmd)) => {
            // 通道满了：把发送任务交给 tokio 等待
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                if let Err(e) = tx_clone.send(cmd).await {
                    eprintln!("delay tx send failed: {e:?}");
                }
            });
        }
        Err(e) => {
            eprintln!("try_send other error: {e:?}");
        }
    }
}

pub(crate) struct DelayExecutor {
    timer: slint::Timer,
    pub(crate) executor: Executor,
    command: Arc<RwLock<VecDeque<Command>>>,
}

impl DelayExecutor {
    pub(crate) fn new(executor: Executor) -> (Self, DelayTX) {
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
                        current_figure.push_back(figure);
                    }

                    // 延迟完成
                    _ = async {
                        if let Some(Command::Figure {delay, ..})
                        | Some(Command::Move {delay, ..}) = current_figure.front(){
                            sleep(Duration::from_millis(
                                delay.clone().unwrap_or_default().parse::<u64>().unwrap_or(0),
                            )).await;
                        } else {
                            std::future::pending::<()>().await
                        }
                    } => {
                        if let Some(cmd) = current_figure.pop_front() {
                            command_clone.write().unwrap().push_back(cmd);
                        }
                    }

                    // 重置请求
                    _ = skip_rx.recv() => {
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

    pub(crate) fn start_timer(&self) {
        let executor = self.executor.clone();
        let command = self.command.clone();

        self.timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(30),
            move || {
                if let Some(mut cmd) = command.write().unwrap().pop_front() {
                    cmd.delete_delay();
                    let result = if let Command::Figure { .. } = &cmd {
                        executor.show_fg(&cmd)
                    } else if let Command::Move { .. } = &cmd {
                        executor.show_move(&cmd)
                    } else {
                        Ok(())
                    };
                    if let Err(e) = result {
                        eprintln!("delay executors failed: {e}");
                    }
                }
            },
        );
    }
}

use crate::error::EngineError;
use crate::executor::auto_executor::AutoExecutor;
use crate::executor::delay_executor::DelayExecutor;
use crate::executor::executor::Executor;
use crate::executor::skip_executor::SkipExecutor;
use tokio::sync::mpsc::Sender;

pub mod auto_executor;
pub mod delay_executor;
pub mod executor;
pub mod skip_executor;

pub struct ExecutorTX {
    auto_tx: Sender<()>,
    skip_tx: Sender<()>,
    _auto_executor: AutoExecutor,
    _skip_executor: SkipExecutor,
    _delay_executor: DelayExecutor,
    _delay_move_executor: DelayExecutor,
}

impl ExecutorTX {
    pub fn auto_tx(&self) -> Sender<()> {
        self.auto_tx.clone()
    }

    pub fn skip_tx(&self) -> Sender<()> {
        self.skip_tx.clone()
    }
}

pub fn load_data(executor: &mut Executor) -> Result<ExecutorTX, EngineError> {
    let (delay_executor, delay_tx, figure_skip_tx, figure_clear_tx) =
        DelayExecutor::new(executor.clone());
    delay_executor.start_timer();
    executor.set_delay_tx(delay_tx);
    executor.set_fg_skip_tx(figure_skip_tx);
    executor.set_fg_clear_tx(figure_clear_tx);

    let (mut delay_move_executor, delay_move_tx, move_skip_tx, move_clear_tx) =
        DelayExecutor::new(executor.clone());
    executor.set_delay_move_tx(delay_move_tx.clone());
    delay_move_executor
        .executor
        .set_delay_move_tx(delay_move_tx);
    executor.set_move_skip_tx(move_skip_tx);
    executor.set_move_clear_tx(move_clear_tx);
    delay_move_executor.start_timer();

    let (mut auto_executor, auto_tx, auto_delay_tx) = AutoExecutor::new(executor.clone());
    executor.set_auto_tx(auto_delay_tx.clone());
    auto_executor.executor.set_auto_tx(auto_delay_tx);
    auto_executor.start_timer();

    let (mut skip_executor, skip_tx) = SkipExecutor::new(executor.clone());
    skip_executor.start_timer();

    executor.load_save_data()?;
    executor.load_volume();
    executor.load_auto();

    Ok(ExecutorTX {
        auto_tx,
        skip_tx,
        _auto_executor: auto_executor,
        _skip_executor: skip_executor,
        _delay_executor: delay_executor,
        _delay_move_executor: delay_move_executor,
    })
}

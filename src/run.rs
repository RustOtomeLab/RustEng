use crate::error::EngineError;
use crate::media::player::Player;
use crate::script::Script;
use crate::ui::initialize::ui;
use std::{cell::RefCell, rc::Rc};

pub async fn build() -> Result<(), EngineError> {
    let mut script = Script::new();
    script.with_name("ky01")?;
    let script = Rc::new(RefCell::new(script));
    ui(script, Player::new()?, Player::new()?).await?;
    Ok(())
}

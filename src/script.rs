use std::collections::VecDeque;
use crate::parser::script_parser::Commands;

#[derive(Debug, Clone)]
pub struct Script {
    commands: VecDeque<Commands>,
}

impl Script {
    pub fn from_commands(commands: VecDeque<Commands>) -> Self {
        Script { commands }
    }
    
    pub fn next_command(&mut self) -> Option<Commands> {
        self.commands.pop_front()
    }
}

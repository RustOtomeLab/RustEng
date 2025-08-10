use crate::error::EngineError;
use crate::parser::script_parser::{parse_script, Commands};
use std::collections::HashMap;
use std::fs;

struct Args {
    path: String,
}

impl Args {
    fn new(args: &str) -> Args {
        Args {
            path: format!("./source/script/{}.reg", args),
        }
    }
}

impl Default for Args {
    fn default() -> Self {
        Args {
            path: "./source/script/ky01.reg".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Script {
    name: String,
    commands: Vec<Commands>,
    current_block: usize,
    labels: HashMap<String, usize>,
}

impl Script {
    pub fn from_name(name: String) -> Result<Self, EngineError> {
        let path = Args::new(&name);
        let script = fs::read_to_string(&path.path)?;
        let (commands, labels) = parse_script(&script, &name)?;
        Ok(Script {
            name,
            commands,
            current_block: 1,
            labels,
        })
    }

    pub fn next_command(&mut self) -> Option<&Commands> {
        let command = self.commands.get(self.current_block);
        self.current_block += 1;
        command
    }

    pub fn set_index(&mut self, index: usize) {
        self.current_block = index;
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn index(&self) -> usize {
        self.current_block
    }

    pub fn find_label(&self, name: &str) -> Option<&usize> {
        self.labels.get(name)
    }
}

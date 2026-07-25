use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CgMap {
    cg: Vec<u64>,
}

impl CgMap {
    pub(crate) fn new(cg: Vec<u64>) -> Self {
        Self { cg }
    }

    pub(crate) fn cg(&self) -> Vec<u64> {
        self.cg.clone()
    }
}
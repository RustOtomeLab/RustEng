use crate::config::voice::{deserialize_duration_from_secs, serialize_duration_as_secs};
use serde::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Deserialize, Serialize)]
pub struct AutoConfig {
    #[serde(
        deserialize_with = "deserialize_duration_from_secs",
        serialize_with = "serialize_duration_as_secs"
    )]
    delay: Duration,
    is_wait: bool,
}

impl AutoConfig {
    pub fn delay(&self) -> Duration {
        self.delay
    }

    pub fn is_wait(&self) -> bool {
        self.is_wait
    }
}

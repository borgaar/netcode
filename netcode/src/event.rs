use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Event {
    pub player_id: usize,
    pub variant: Variant
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Variant {
    Jump(JumpEvent),
    Movement(f64)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JumpEvent {
    pub at: chrono::DateTime<Utc>,
}
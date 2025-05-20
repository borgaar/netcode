use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Action {
    pub player_id: usize,
    pub variant: Variant
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Variant {
    Join,
    Jump(JumpAction),
    Movement(f64)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JumpAction {
    pub at: chrono::DateTime<Utc>,
}
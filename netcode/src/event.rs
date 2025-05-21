use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Action {
    Join,
    Player { id: usize, action: PlayerAction },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum PlayerAction {
    Jump { at: chrono::DateTime<Utc> },
    Move { delta_x: f64 },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JoinResponse {
    pub player_id: usize,
}

impl JoinResponse {
    pub fn new(player_id: usize) -> Self {
        Self { player_id }
    }
}

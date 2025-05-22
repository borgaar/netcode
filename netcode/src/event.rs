use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Action {
    Join,
    Player { id: usize, action: PlayerAction },
}

impl Action {
    pub fn player_join() -> Self {
        Self::Join
    }

    pub fn player_jump(player_id: usize, at: chrono::DateTime<Utc>) -> Self {
        Self::Player {
            id: player_id,
            action: PlayerAction::Jump { at },
        }
    }

    pub fn player_move(player_id: usize, delta_x: f64) -> Self {
        Self::Player {
            id: player_id,
            action: PlayerAction::Move {
                delta_x,
                id: Uuid::new_v4(),
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum PlayerAction {
    Jump { at: chrono::DateTime<Utc> },
    Move { delta_x: f64, id: Uuid },
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

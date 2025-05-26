use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Action that can be sent to the server's [ACTION] channel
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Action {
    Join,
    Player { id: usize, action: PlayerAction },
}

impl Action {
    /// Join the game
    pub fn player_join() -> Self {
        Self::Join
    }

    /// Get the ack_id of the current event, if it has one
    pub fn ack_id(&self) -> Option<(Uuid, PlayerAction)> {
        match self {
            Action::Join => None,
            Action::Player { action, id: _ } => match action {
                PlayerAction::Jump { at: _ } => None,
                PlayerAction::Move { delta_x, id } => Some((
                    *id,
                    PlayerAction::Move {
                        delta_x: *delta_x,
                        id: *id,
                    },
                )),
            },
        }
    }

    /// Create a jump action
    pub fn player_jump(player_id: usize, at: chrono::DateTime<Utc>) -> Self {
        Self::Player {
            id: player_id,
            action: PlayerAction::Jump { at },
        }
    }

    /// Create a player movement action
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

/// Actions that can be performed on a player that has joined the game
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum PlayerAction {
    Jump { at: chrono::DateTime<Utc> },
    Move { delta_x: f64, id: Uuid },
}

/// Response from joining the game; includes the player's global ID
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JoinResponse {
    pub player_id: usize,
}

impl JoinResponse {
    /// Create a join response
    pub fn new(player_id: usize) -> Self {
        Self { player_id }
    }
}

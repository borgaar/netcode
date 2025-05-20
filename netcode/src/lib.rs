use chrono::{DateTime, Utc};

pub mod client;
pub mod event;
pub mod server;

pub use event::Action;
use serde::{Deserialize, Serialize};

pub const ACTION_CHANNEL: &str = "action";
pub const STATE_CHANNEL: &str = "state";
pub const JOIN_CHANNEL: &str = "join";
pub const ERROR_CHANNEL: &str = "error";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct State {
    pub players: Vec<Player>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Player {
    pub id: usize,
    pub x: f64,
    pub last_jump_at: chrono::DateTime<Utc>,
}

impl Player {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            x: 0.0,
            last_jump_at: DateTime::from_timestamp_nanos(0),
        }
    }

    pub fn y(&self) -> f64 {
        let t = (chrono::Utc::now() - self.last_jump_at).as_seconds_f64();

        if t < 0.0 || t > 0.33 {
            0.0
        } else {
            -(3.0 * t).powi(2) + 3.0 * t
        }
    }
}

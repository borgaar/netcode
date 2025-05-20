use std::sync::Arc;

use chrono::Utc;

pub mod client;
pub mod event;
pub mod server;

pub use event::Event;

pub struct State {
    pub players: Vec<Player>,
}

pub struct Player {
    pub id: usize,
    pub x: f32,
    pub last_jump_at: chrono::DateTime<Utc>,
}

impl Player {
    fn y(&self) -> f64 {
        let t = (chrono::Utc::now() - self.last_jump_at).as_seconds_f64();

        if t < 0.0 || t > 1.0 {
            0.0
        } else {
            -(3.0 * t).powi(2) + 3.0 * t
        }
    }
}


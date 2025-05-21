use crate::MAX_UNITS_PER_SECOND;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct State {
    pub players: Vec<Player>,
    pub timestamp: DateTime<Utc>,
}

#[derive(thiserror::Error, Debug)]
pub enum StateError {
    #[error("No player found with id: {0}. Total players is {1}")]
    NoPlayer(usize, usize),
    #[error("Player moved {units} units in the last {timeframe_seconds:.5} s {0:.5} unit/s. Expected at most {MAX_UNITS_PER_SECOND} unit/s", units / timeframe_seconds)]
    Cheating { units: f64, timeframe_seconds: f64 },
}

impl State {
    pub fn tick(&mut self) {
        self.timestamp = Utc::now();
    }

    fn player(&mut self, player_id: usize) -> Result<&mut Player, StateError> {
        let len = self.players.len();
        self.players
            .get_mut(player_id)
            .ok_or(StateError::NoPlayer(player_id, len))
    }

    pub fn player_jump(&mut self, player_id: usize, at: DateTime<Utc>) -> Result<(), StateError> {
        self.player(player_id)?.last_jump_at = Some(at);
        Ok(())
    }

    pub fn player_move(&mut self, player_id: usize, delta_x: f64) -> Result<(), StateError> {
        let seconds_since_last_update = (Utc::now() - self.timestamp).as_seconds_f64();
        let x_per_second = delta_x / seconds_since_last_update as f64;

        if x_per_second.abs() > MAX_UNITS_PER_SECOND {
            self.player(player_id)?.x +=
                MAX_UNITS_PER_SECOND * if x_per_second.is_sign_positive() { 1.0 } else { -1.0 };
                
            return Err(StateError::Cheating {
                units: delta_x,
                timeframe_seconds: seconds_since_last_update,
            });
        }

        self.player(player_id)?.x += delta_x;
        Ok(())
    }

    pub fn player_join(&mut self) -> usize {
        let len = self.players.len();
        let player = Player::new(len);
        self.players.push(player);
        len
    }

    pub fn player_leave(&mut self, player_id: usize) -> Result<(), StateError> {
        let len = self.players.len();
        if player_id >= len {
            return Err(StateError::NoPlayer(player_id, len));
        }

        self.players.remove(player_id);
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Player {
    pub id: usize,
    pub x: f64,
    pub last_jump_at: Option<chrono::DateTime<Utc>>,
}

impl Player {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            x: 0.0,
            last_jump_at: None,
        }
    }

    pub fn y(&self) -> f64 {
        let Some(last_jump_at) = self.last_jump_at else {
            return 0.0;
        };

        let t = (chrono::Utc::now() - last_jump_at).as_seconds_f64();

        if !(0.0..=0.33).contains(&t) {
            0.0
        } else {
            -(3.0 * t).powi(2) + 3.0 * t
        }
    }
}

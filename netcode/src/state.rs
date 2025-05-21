use std::collections::HashMap;

use crate::MAX_UNITS_PER_SECOND;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub players: HashMap<usize,Player>,
    pub timestamp: DateTime<Utc>,
    #[serde(skip)]
    new_player_id: usize
}

impl Default for State {
    fn default() -> Self {
        Self {
            players: HashMap::new(),
            timestamp: Utc::now(),
            new_player_id: 0
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum StateError {
    #[error("[ERROR - UNKNOWN PLAYER] No player found with id: {0}.")]
    UnknownPlayer(usize),
    #[error("[ERROR - CHEATING] Player moved {units} units in the last {timeframe_seconds:.5} s ({0:.5} unit/s). Expected at most {MAX_UNITS_PER_SECOND} unit/s", units / timeframe_seconds)]
    Cheating { units: f64, timeframe_seconds: f64 },
}

impl State {
    pub fn tick(&mut self) {
        self.timestamp = Utc::now();
    }

    fn player(&mut self, player_id: usize) -> Result<&mut Player, StateError> {
        self.players
            .get_mut(&player_id)
            .ok_or(StateError::UnknownPlayer(player_id))
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
        let id = self.new_player_id;
        self.new_player_id += 1;
        let player = Player::new(id);
        self.players.insert(id, player);
        id
    }

    pub fn player_leave(&mut self, player_id: usize) -> Result<(), StateError> {
        match self.players.remove(&player_id) {
            Some(_) => Ok(()),
            None => Err(StateError::UnknownPlayer(player_id)),
        }
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

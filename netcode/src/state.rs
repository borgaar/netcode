use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const MAX_UNITS_PER_SECOND: f64 = 2.5;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct State {
    pub players: Vec<Player>,
}

#[derive(thiserror::Error, Debug)]
pub enum StateError {
    #[error("No player found with id: {0}. Total players is {1}")]
    NoPlayer(usize, usize),
    #[error("Player moved {0} units. Expected at most {MAX_UNITS_PER_SECOND}")]
    Cheating(f64),
}

impl State {
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

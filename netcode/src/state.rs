use std::collections::{HashMap, HashSet};

use crate::MAX_UNITS_PER_SECOND;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Server state including position of all players and tick info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub players: HashMap<usize, Player>,
    pub timestamp: DateTime<Utc>,
    #[serde(skip)]
    new_player_id: usize,
    pub acknowledged: HashSet<Uuid>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            players: HashMap::new(),
            timestamp: Utc::now(),
            new_player_id: 0,
            acknowledged: HashSet::new(),
        }
    }
}

/// Represents all possible errors that can occur when updating the game state
#[derive(thiserror::Error, Debug)]
pub enum StateError {
    #[error("[ERROR - UNKNOWN PLAYER] No player found with id: {0}.")]
    UnknownPlayer(usize),
    #[error("[ERROR - CHEATING] Player moved {units:.5} units in the last {timeframe_seconds:.5} s ({0:.5} unit/s). Expected at most {MAX_UNITS_PER_SECOND} unit/s", units / timeframe_seconds)]
    Cheating { units: f64, timeframe_seconds: f64 },
}

impl State {
    /// Update the timestamp of the game's last update
    pub fn tick(&mut self) -> String {
        self.timestamp = Utc::now();
        let message = serde_json::to_string_pretty(self).unwrap();
        self.clear_ack();
        message
    }

    /// Clears acknowledged requests that are part of the current state,
    /// to ensure no duplicate requests
    pub fn clear_ack(&mut self) {
        self.acknowledged.clear();
    }

    /// Get a player by id. Returns [StateError::UnknownPlayer] if the player does not exist.
    fn player(&mut self, player_id: usize) -> Result<&mut Player, StateError> {
        self.players
            .get_mut(&player_id)
            .ok_or(StateError::UnknownPlayer(player_id))
    }

    /// Makes a player jump at the specified time.
    /// Time is sent by the frontend to ensure consistent jump animation in real-time.
    pub fn player_jump(&mut self, player_id: usize, at: DateTime<Utc>) -> Result<(), StateError> {
        self.player(player_id)?.last_jump_at = Some(at);
        Ok(())
    }

    /// Makes a player move, including cheat protection if the move is too long since the last game update
    pub fn player_move(
        &mut self,
        player_id: usize,
        delta_x: f64,
        ack_id: Uuid,
    ) -> Result<(), StateError> {
        // let seconds_since_last_update = (Utc::now() - self.timestamp).as_seconds_f64();
        // let x_per_second = delta_x / seconds_since_last_update as f64;

        // if x_per_second.abs() > MAX_UNITS_PER_SECOND {
        //     self.player(player_id)?.x +=
        //         MAX_UNITS_PER_SECOND * if x_per_second.is_sign_positive() { 1.0 } else { -1.0 };

        //     return Err(StateError::Cheating {
        //         units: delta_x,
        //         timeframe_seconds: seconds_since_last_update,
        //     });
        // }

        self.acknowledged.insert(ack_id);
        self.player(player_id)?.x += delta_x;
        Ok(())
    }

    /// Makes a player join the game, returning the player's ID
    pub fn player_join(&mut self) -> usize {
        let id = self.new_player_id;
        self.new_player_id += 1;
        let player = Player::new(id);
        self.players.insert(id, player);
        id
    }

    /// Makes a player leave the game, returning a [StateError] if the player does not exist.
    pub fn player_leave(&mut self, player_id: usize) -> Result<(), StateError> {
        match self.players.remove(&player_id) {
            Some(_) => Ok(()),
            None => Err(StateError::UnknownPlayer(player_id)),
        }
    }
}

/// A single player in the game, with id and positional info
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Player {
    pub id: usize,
    pub x: f64,
    pub last_jump_at: Option<chrono::DateTime<Utc>>,
}

impl Player {
    /// Create a player by ID
    pub fn new(id: usize) -> Self {
        Self {
            id,
            x: 0.0,
            last_jump_at: None,
        }
    }

    /// Get the player's current Y coordinate given the player's state
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

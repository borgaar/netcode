use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver},
    thread, usize,
};

use chrono::{TimeDelta, Utc};
use rust_socketio::{client::Client, ClientBuilder, Payload};
use uuid::Uuid;

use crate::{
    event::{JoinResponse, PlayerAction},
    state::Player,
    Action, State, ACTION_CHANNEL, ERROR_CHANNEL, JOIN_CHANNEL, MAX_UNITS_PER_SECOND,
    STATE_CHANNEL,
};

const SIMULATED_SEND_PING_MS: u64 = 150;

pub struct Game {
    state_receiver: Receiver<State>,
    join_receiver: Receiver<JoinResponse>,
    pub curr_state: State,
    target_state: State,
    previous_state: State,
    pub player_idx: Option<usize>,
    client: Client,
    unacknowledged: HashMap<Uuid, PlayerAction>,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Self {
        let (state_sender, state_receiver) = channel::<State>();
        let (join_sender, join_receiver) = channel::<JoinResponse>();

        let game = Self {
            state_receiver,
            join_receiver,
            unacknowledged: HashMap::new(),
            curr_state: Default::default(),
            previous_state: Default::default(),
            target_state: Default::default(),
            player_idx: None,
            client: ClientBuilder::new("http://localhost:7878")
                .on(ERROR_CHANNEL, |payload, _| {
                    let Payload::Text(val) = payload else {
                        return;
                    };
                    eprintln!("{}", val.first().unwrap().as_str().unwrap());
                })
                .on(STATE_CHANNEL, move |payload, _| match payload {
                    Payload::Text(text) => {
                        let data = serde_json::from_str::<State>(
                            text.first().unwrap().clone().as_str().unwrap(),
                        )
                        .unwrap();
                        let sender = state_sender.clone();
                        thread::spawn(move || {
                            thread::sleep(std::time::Duration::from_millis(SIMULATED_SEND_PING_MS));
                            sender.send(data).unwrap();
                        });
                    }
                    _ => {
                        eprintln!("Received bad payload on state");
                    }
                })
                .on(JOIN_CHANNEL, move |payload, _| match payload {
                    Payload::Text(text) => {
                        let data = serde_json::from_str::<JoinResponse>(
                            text.first().unwrap().clone().as_str().unwrap(),
                        )
                        .unwrap();
                        let sender = join_sender.clone();
                        thread::spawn(move || {
                            thread::sleep(std::time::Duration::from_millis(SIMULATED_SEND_PING_MS));
                            sender.send(data).unwrap();
                        });
                    }
                    _ => {
                        eprintln!(
                            "Received non-binary payload on join, received {:?}",
                            payload
                        );
                    }
                })
                .connect()
                .unwrap(),
        };

        game
    }
    pub fn join(&self) {
        self.client
            .emit(
                ACTION_CHANNEL,
                Payload::Text(vec![serde_json::to_value(&Action::Join).unwrap()]),
            )
            .unwrap();
    }
    pub fn update(&mut self) {
        self.state_update();
        self.join_update();
    }

    fn calculate_interpolation_for_frame(&mut self) {
        let prev = self.previous_state.timestamp;
        let target = self.target_state.timestamp;
        let curr = Utc::now();
        let t = (curr - target).as_seconds_f64() / (target - prev).as_seconds_f64();

        let player_id = self.player_idx.unwrap_or(usize::MAX);

        let self_player = self
            .curr_state
            .players
            .get(&player_id)
            .or_else(|| self.target_state.players.get(&player_id));

        let new_curr_players = self
            .target_state
            .players
            .iter()
            .map(|(tar_player_id, tar_player)| {
                if *tar_player_id == player_id && self_player.is_some() {
                    return (self_player.unwrap().id, self_player.unwrap().clone());
                }

                let prev_player = self
                    .previous_state
                    .players
                    .get(&tar_player_id)
                    .unwrap_or_else(|| tar_player);

                let x = lerp(prev_player.x, tar_player.x, t);

                return (
                    *tar_player_id,
                    Player {
                        id: *tar_player_id,
                        x,
                        last_jump_at: tar_player.last_jump_at,
                    },
                );
            })
            .collect::<HashMap<_, _>>();

        self.curr_state.players = new_curr_players;
    }

    fn player(&self) -> Option<&Player> {
        let Some(player_idx) = self.player_idx else {
            return None;
        };

        self.curr_state.players.get(&player_idx)
    }

    fn state_update(&mut self) {
        self.calculate_interpolation_for_frame();

        for server_state in self.state_receiver.try_iter() {
            // server update
            self.previous_state = self.target_state.clone();
            self.target_state = server_state.clone();

            let Some(player_idx) = self.player_idx else {
                continue;
            };

            let Some(_) = self.player() else {
                continue;
            };

            let time_since_last_update = (Utc::now() - self.curr_state.timestamp).as_seconds_f64();

            // Remove acknowledged actions
            self.unacknowledged
                .retain(|key, _| server_state.acknowledged.get(key).is_none());

            dbg!(&server_state.acknowledged);
            dbg!(&self
                .unacknowledged
                .iter()
                .map(|(key, _)| key)
                .collect::<Vec<_>>());
            println!("\n\n\n");

            // Get the unacknowledged diff from the last update
            let unack_x_diff = self.get_unack_x_diff();

            let new_relative_position =
                server_state.players.get(&player_idx).unwrap().x + unack_x_diff;

            let x_diff = self.player().unwrap().x - new_relative_position;

            let max_diff_x = time_since_last_update * MAX_UNITS_PER_SECOND;

            let effective_diff_x = if x_diff.abs() > max_diff_x.abs() {
                if x_diff.is_sign_negative() {
                    -max_diff_x
                } else {
                    max_diff_x
                }
            } else {
                x_diff
            };

            if effective_diff_x.abs() <= 1.0 {
                return;
            }

            let server_side_x = server_state
                .players
                .get(&self.player().unwrap().id)
                .unwrap()
                .x;

            // Reconciliation
            self.curr_state
                .players
                .get_mut(&self.player_idx.unwrap())
                .unwrap()
                .x = server_side_x + effective_diff_x + unack_x_diff;

            // Send the move action
            let action = Action::player_move(player_idx, effective_diff_x);

            self.client
                .emit(
                    ACTION_CHANNEL,
                    Payload::Text(vec![serde_json::to_value(&action).unwrap()]),
                )
                .unwrap();

            // Add the action to the unacknowledged actions
            if let Some((id, action)) = action.ack_id() {
                self.unacknowledged.insert(id, action);
            }
        }
    }

    fn get_unack_x_diff(&self) -> f64 {
        let mut x_diff = 0.0;
        for action in self.unacknowledged.values() {
            match action {
                PlayerAction::Move { delta_x, id } => {
                    x_diff += *delta_x;
                }
                PlayerAction::Jump { at: _ } => {}
            }
        }
        x_diff
    }

    fn join_update(&mut self) {
        for join_response in self.join_receiver.try_iter() {
            self.player_idx = Some(join_response.player_id);
            self.curr_state.players.insert(
                join_response.player_id,
                Player::new(join_response.player_id),
            );
            self.target_state.players.insert(
                join_response.player_id,
                Player::new(join_response.player_id),
            );
        }
    }
    pub fn jump(&mut self) {
        if let Some(player_idx) = self.player_idx {
            // Optimistic update
            self.curr_state
                .players
                .get_mut(&player_idx)
                .unwrap()
                .last_jump_at = Some(chrono::Utc::now());

            self.client
                .emit(
                    ACTION_CHANNEL,
                    Payload::Text(vec![serde_json::to_value(&Action::Player {
                        id: player_idx,
                        action: PlayerAction::Jump { at: Utc::now() },
                    })
                    .unwrap()]),
                )
                .unwrap();
        }
    }
    pub fn move_player(&mut self, delta_x: f32) {
        let Some(player_idx) = self.player_idx else {
            return;
        };

        // Optimistic update
        self.curr_state.players.get_mut(&player_idx).unwrap().x += delta_x as f64;
    }
}

/// Linear interpolation between two values
///
/// * `a` - The start value
/// * `b` - The end value
/// * `t` - The interpolation factor (between 0.0 and 1.0)
///
/// Returns the interpolated value: a + t * (b - a)
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + t * (b - a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0., 1., 0.5), 0.5);
        assert_eq!(lerp(0., 10., 0.25), 2.5);
    }
}

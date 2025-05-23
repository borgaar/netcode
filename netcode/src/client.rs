use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread, usize,
};

use chrono::{TimeDelta, Utc};
use rust_socketio::{client::Client, ClientBuilder, Payload};
use uuid::Uuid;

use crate::{
    event::{JoinResponse, PlayerAction},
    state::Player,
    Action, State, ACTION_CHANNEL, ERROR_CHANNEL, JOIN_CHANNEL, STATE_CHANNEL,
};

pub struct Game {
    state_receiver: Receiver<State>,
    join_receiver: Receiver<JoinResponse>,
    pub display_state: State,
    target_state: State,
    previous_state: State,
    pub player_idx: Option<usize>,
    client: Client,
    unacknowledged: HashMap<Uuid, PlayerAction>,
    simulated_ping: Arc<Mutex<u64>>,
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

        let simulated_ping = Arc::new(Mutex::new(0));

        let game = Self {
            state_receiver,
            join_receiver,
            unacknowledged: HashMap::new(),
            display_state: Default::default(),
            previous_state: Default::default(),
            target_state: Default::default(),
            player_idx: None,
            client: build_netcode_client(state_sender, join_sender, simulated_ping.clone()),
            simulated_ping,
        };

        game
    }
    pub fn join(&self) {
        if let Err(_) = self.client.emit(
            ACTION_CHANNEL,
            Payload::Text(vec![serde_json::to_value(&Action::Join).unwrap()]),
        ) {
            eprintln!("Failed to join the game");
        };
    }
    pub fn update(&mut self) {
        self.state_update();
        self.join_update();
    }

    fn calculate_interpolation_for_frame(&mut self) {
        let prev = self.previous_state.timestamp;
        let target = self.target_state.timestamp;
        let curr =
            Utc::now() - TimeDelta::milliseconds((*self.simulated_ping.lock().unwrap() / 2) as i64);
        let t = (curr - target).as_seconds_f64() / (target - prev).as_seconds_f64();

        let player_id = self.player_idx.unwrap_or(usize::MAX);

        let self_player = self
            .display_state
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

        self.display_state.players = new_curr_players;
    }

    fn get_player(&self) -> Option<&Player> {
        let player_id = self.player_idx?;
        self.display_state.players.get(&player_id)
    }

    fn state_update(&mut self) {
        for server_state in self.state_receiver.try_iter() {
            self.previous_state = self.target_state.clone();
            self.target_state = server_state.clone();

            // Get the current player
            let current_player = match self.get_player() {
                Some(player) => player.clone(),
                None => continue,
            };

            // Print length of server play list
            dbg!(server_state.players.len());

            // print length of unacknowledged actions
            dbg!(self.unacknowledged.len());

            // Remove acknowledged actions
            self.unacknowledged
                .retain(|key, _| server_state.acknowledged.get(key).is_none());

            // Get the unacknowledged diff from the last update
            let unacknowledged_x_diff = self.get_unack_x_diff();

            // Get the server's position for this player
            let Some(server_player) = server_state.players.get(&current_player.id) else {
                continue;
            };

            // The server's position plus what has been unacknowledged (sent but not yet processed)
            let reconciled_position = server_player.x + unacknowledged_x_diff;

            // movement since last sent position change
            let position_discrepancy = current_player.x - reconciled_position;

            // Get the jump time from display state
            let local_last_jump_at = current_player
                .last_jump_at
                .unwrap_or_else(|| Utc::now() - TimeDelta::milliseconds(1000));

            // Update the display state with the state from the server
            self.display_state = server_state.clone();

            // Update the display position and jump time to the reconciled+predicted position
            if let Some(player) = self.display_state.players.get_mut(&current_player.id) {
                player.x = reconciled_position + position_discrepancy;
                player.last_jump_at = Some(local_last_jump_at);
                dbg!(player.x);
                dbg!(server_state.players.get(&current_player.id).unwrap().x);
                dbg!(reconciled_position);
                dbg!(position_discrepancy);
                println!("\n\n\n\n\n")
            }

            // Check if the discrepancy is significant enough to send new move
            if position_discrepancy.abs() < 0.01 {
                return;
            }

            let action = Action::player_move(current_player.id, position_discrepancy);

            // Add the action to the unacknowledged actions and send
            if let Some((id, player_action)) = action.ack_id() {
                let client_clone = self.client.clone();
                let ping = self.simulated_ping.clone();
                thread::spawn(move || {
                    thread::sleep(std::time::Duration::from_millis(*ping.lock().unwrap() / 2));
                    client_clone
                        .emit(
                            ACTION_CHANNEL,
                            Payload::Text(vec![serde_json::to_value(&action).unwrap()]),
                        )
                        .unwrap();
                });
                self.unacknowledged.insert(id, player_action);
            }
        }

        self.calculate_interpolation_for_frame();
    }

    fn get_unack_x_diff(&self) -> f64 {
        let mut x_diff = 0.0;
        for action in self.unacknowledged.values() {
            match action {
                PlayerAction::Move { delta_x, id: _ } => {
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
            self.display_state.players.insert(
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
            self.display_state
                .players
                .get_mut(&player_idx)
                .unwrap()
                .last_jump_at = Some(chrono::Utc::now());

            let client_clone = self.client.clone();
            let ping = self.simulated_ping.clone();
            thread::spawn(move || {
                thread::sleep(std::time::Duration::from_millis(*ping.lock().unwrap() / 2));
                client_clone
                    .emit(
                        ACTION_CHANNEL,
                        Payload::Text(vec![serde_json::to_value(&Action::Player {
                            id: player_idx,
                            action: PlayerAction::Jump { at: Utc::now() },
                        })
                        .unwrap()]),
                    )
                    .unwrap();
            });
        }
    }
    pub fn move_player(&mut self, delta_x: f32) {
        let Some(player_idx) = self.player_idx else {
            return;
        };

        // Optimistic update
        self.display_state.players.get_mut(&player_idx).unwrap().x += delta_x as f64;
    }

    pub fn set_simulated_ping(&self, new_ping: u64) -> u64 {
        let mut ping = self.simulated_ping.lock().unwrap();
        *ping = new_ping;
        *ping
    }
}

fn build_netcode_client(
    state_sender: Sender<State>,
    join_sender: Sender<JoinResponse>,
    ping: Arc<Mutex<u64>>,
) -> Client {
    let state_ping = ping.clone();
    let join_ping = ping.clone();
    ClientBuilder::new("http://localhost:7878")
        .on(ERROR_CHANNEL, |payload, _| {
            let Payload::Text(val) = payload else {
                return;
            };
            eprintln!("{}", val.first().unwrap().as_str().unwrap());
        })
        .on(STATE_CHANNEL, move |payload, _| match payload {
            Payload::Text(text) => {
                let data =
                    serde_json::from_str::<State>(text.first().unwrap().clone().as_str().unwrap())
                        .unwrap();
                let sender = state_sender.clone();
                let ping = state_ping.clone();
                thread::spawn(move || {
                    thread::sleep(std::time::Duration::from_millis(*ping.lock().unwrap() / 2));
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
                let ping = join_ping.clone();
                thread::spawn(move || {
                    thread::sleep(std::time::Duration::from_millis(*ping.lock().unwrap() / 2));
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
        .unwrap()
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

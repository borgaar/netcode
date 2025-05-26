//! Handles client side state updates with reconciliation, interpolation and prediction.

use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use chrono::{TimeDelta, Utc};
use rust_socketio::{client::Client, ClientBuilder, Payload};
use uuid::Uuid;

use crate::{
    event::{JoinResponse, PlayerAction},
    state::Player,
    Action, State, ACTION_CHANNEL, ERROR_CHANNEL, JOIN_CHANNEL, STATE_CHANNEL,
};

/// Game state that is mutated through the lifecycle of the client.
pub struct Game {
    state_receiver: Receiver<State>,
    join_receiver: Receiver<JoinResponse>,
    pub local_state: State,
    target_state: State,
    previous_state: State,
    pub display_state: State,
    pub player_idx: Option<usize>,
    client: Client,
    pub unacknowledged: HashMap<Uuid, PlayerAction>,
    simulated_ping: Arc<Mutex<u64>>,
    pub ping_cache: u64,
    pub prediction: bool,
    pub reconciliation: bool,
    pub interpolation: bool,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    /// Creates a game state with default values
    pub fn new() -> Self {
        let (state_sender, state_receiver) = channel::<State>();
        let (join_sender, join_receiver) = channel::<JoinResponse>();

        let simulated_ping = Arc::new(Mutex::new(250));

        Self {
            state_receiver,
            join_receiver,
            unacknowledged: HashMap::new(),
            local_state: Default::default(),
            previous_state: Default::default(),
            target_state: Default::default(),
            display_state: Default::default(),
            player_idx: None,
            client: build_netcode_client(state_sender, join_sender, simulated_ping.clone()),
            simulated_ping,
            ping_cache: 0,
            prediction: true,
            reconciliation: true,
            interpolation: true,
        }
    }

    /// Join the server-side game.
    /// Make sure to only call this function once, as any future calls result in multiple sessions.
    pub fn join(&self) {
        if self
            .client
            .emit(
                ACTION_CHANNEL,
                Payload::Text(vec![serde_json::to_value(&Action::Join).unwrap()]),
            )
            .is_err()
        {
            eprintln!("Failed to join the game");
        };
    }

    /// Tick the game's state. Should be called every frame.
    pub fn update(&mut self) {
        self.state_update();
        self.join_update();
    }

    /// Handles calculating other player's current coordinates based on the current state.
    fn calculate_interpolation_for_frame(&mut self) {
        // Find time the lerping value, t, for interpolation using previously obtained states
        let prev = self.previous_state.timestamp;
        let target = self.target_state.timestamp;
        let curr = Utc::now() - TimeDelta::milliseconds((self.ping_cache / 2) as i64);
        let t = (curr - target).as_seconds_f64() / (target - prev).as_seconds_f64();

        // Due to prediction and reconciliation we will handle own player differently
        let player_id = self.player_idx.unwrap_or(usize::MAX);

        let self_player = self
            .local_state
            .players
            .get(&player_id)
            .or_else(|| self.target_state.players.get(&player_id));

        let mut new_curr_players = self
            .target_state
            .players
            .iter()
            .map(|(tar_player_id, tar_player)| {
                if *tar_player_id == player_id {
                    if let Some(self_player) = self_player {
                        return (*tar_player_id, self_player.clone());
                    }
                }

                let prev_player = self
                    .previous_state
                    .players
                    .get(tar_player_id)
                    .unwrap_or(tar_player);

                // Find x value based on linear interpolation between previous and target player
                // positions
                let x = lerp(prev_player.x, tar_player.x, t);

                (
                    *tar_player_id,
                    Player {
                        id: *tar_player_id,
                        x,
                        last_jump_at: tar_player.last_jump_at,
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        // Update position on working local state
        self.local_state.players = new_curr_players.clone();
        if let Some(display_player) = self.display_state.players.get(&player_id) {
            // Add the current reconciled/predicted player's position to the list of lerped players
            new_curr_players.insert(player_id, display_player.clone());
        }
        // Update the display state with the new lerped players
        self.display_state.players = new_curr_players;
    }

    /// Get the current player
    /// May be [None] if no current player (not yet joined)
    fn get_player(&self) -> Option<&Player> {
        let player_id = self.player_idx?;
        self.local_state.players.get(&player_id)
    }

    /// Handles updating the state of an active game.
    fn state_update(&mut self) {
        for server_state in self.state_receiver.try_iter() {
            self.previous_state = self.target_state.clone();
            self.target_state = server_state.clone();

            // Update ping cache
            {
                self.ping_cache = *self.simulated_ping.lock().unwrap();
            }

            // Get the current player
            let current_player = match self.get_player() {
                Some(player) => player.clone(),
                None => {
                    // Update local and display state and continue
                    self.local_state = server_state.clone();
                    self.display_state = server_state;
                    continue;
                }
            };

            // Remove acknowledged actions
            self.unacknowledged
                .retain(|key, _| !server_state.acknowledged.contains(key));

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

            // Update the local and display state with the state from the server
            self.local_state = server_state.clone();
            self.display_state = self.previous_state.clone();

            // Update the display position and jump time to the reconciled+predicted position
            if let Some(player) = self.local_state.players.get_mut(&current_player.id) {
                player.x = reconciled_position + position_discrepancy;
                player.last_jump_at = Some(local_last_jump_at);

                if self.reconciliation {
                    if let Some(display_player) =
                        self.display_state.players.get_mut(&current_player.id)
                    {
                        display_player.x = reconciled_position + position_discrepancy;
                        display_player.last_jump_at = Some(local_last_jump_at);
                    }
                }
            }

            // Check if the discrepancy is significant enough to send new move
            if position_discrepancy.abs() < 0.01 {
                return;
            }

            let action = Action::player_move(current_player.id, position_discrepancy);

            // Add the action to the unacknowledged actions and send
            if let Some((id, player_action)) = action.ack_id() {
                let client_clone = self.client.clone();
                let ping_cache = self.ping_cache;
                thread::spawn(move || {
                    thread::sleep(std::time::Duration::from_millis(ping_cache / 2));
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

        if self.interpolation {
            self.calculate_interpolation_for_frame();
        }
    }

    /// Get the total delta_x not accounted for by the received server state.
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

    /// Checks if a join response is available to join the game.
    fn join_update(&mut self) {
        for join_response in self.join_receiver.try_iter() {
            self.player_idx = Some(join_response.player_id);
            self.local_state.players.insert(
                join_response.player_id,
                Player::new(join_response.player_id),
            );
            self.target_state.players.insert(
                join_response.player_id,
                Player::new(join_response.player_id),
            );
        }
    }

    /// Make the current player jump.
    pub fn jump(&mut self) {
        if let Some(player_idx) = self.player_idx {
            // Optimistic update
            self.local_state
                .players
                .get_mut(&player_idx)
                .unwrap()
                .last_jump_at = Some(chrono::Utc::now());

            if self.prediction {
                self.display_state
                    .players
                    .get_mut(&player_idx)
                    .unwrap()
                    .last_jump_at = Some(chrono::Utc::now());
            }

            let client_clone = self.client.clone();

            // Spawn thread to simulate network delay
            let ping_cache = self.ping_cache;
            thread::spawn(move || {
                thread::sleep(std::time::Duration::from_millis(ping_cache / 2));
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

    /// Makes the current player move by [delta_x] units
    pub fn move_player(&mut self, delta_x: f32) {
        let Some(player_idx) = self.player_idx else {
            return;
        };

        // Optimistic update
        self.local_state.players.get_mut(&player_idx).unwrap().x += delta_x as f64;
        if self.prediction {
            if let Some(display_player) = self.display_state.players.get_mut(&player_idx) {
                display_player.x += delta_x as f64;
            }
        }
    }

    /// Update the game's simulated ping amount to check for network issues.
    pub fn set_simulated_ping(&self, new_ping: u64) -> u64 {
        let mut ping = self.simulated_ping.lock().unwrap();
        *ping = new_ping;
        *ping
    }
}

/// Build a client to handle incomming messages from the server.
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
                    let ping = { *ping.lock().unwrap() };

                    thread::sleep(std::time::Duration::from_millis(ping / 2));
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

                let ping = { *ping.lock().unwrap() };

                thread::spawn(move || {
                    thread::sleep(std::time::Duration::from_millis(ping / 2));
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

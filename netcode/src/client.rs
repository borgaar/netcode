use std::sync::mpsc::{channel, Receiver};

use chrono::Utc;
use rust_socketio::{client::Client, ClientBuilder, Payload};

use crate::{
    event::{JoinResponse, PlayerAction},
    state::Player,
    Action, State, ACTION_CHANNEL, ERROR_CHANNEL, JOIN_CHANNEL, MAX_UNITS_PER_SECOND,
    STATE_CHANNEL,
};

pub struct Game {
    state_receiver: Receiver<State>,
    join_receiver: Receiver<JoinResponse>,
    pub state: State,
    pub player_idx: Option<usize>,
    client: Client,
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
            state: Default::default(),
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
                        state_sender.send(data).unwrap();
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
                        join_sender.send(data).unwrap();
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
        println!("Joining");
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
    fn state_update(&mut self) {
        while let Ok(server_state) = self.state_receiver.try_recv() {
            if let Some(player_idx) = self.player_idx {
                let time_since_last_update = (Utc::now() - self.state.timestamp).as_seconds_f64();

                let player_x = self.state.players.get(&player_idx).unwrap().x;

                let diff_x = server_state.players.get(&player_idx).unwrap().x - player_x;

                let max_diff_x = time_since_last_update * MAX_UNITS_PER_SECOND;

                let effective_diff_x = if diff_x.abs() > max_diff_x.abs() {
                    if diff_x.is_sign_negative() {
                        -max_diff_x
                    } else {
                        max_diff_x
                    }
                } else {
                    diff_x
                };

                self.state = server_state;

                self.state.players.get_mut(&player_idx).unwrap().x += effective_diff_x;

                // Send the move action
                self.client
                    .emit(
                        ACTION_CHANNEL,
                        Payload::Text(vec![serde_json::to_value(&Action::Player {
                            id: player_idx,
                            action: PlayerAction::Move { delta_x: diff_x },
                        })
                        .unwrap()]),
                    )
                    .unwrap();
            }
        }
    }
    fn join_update(&mut self) {
        while let Ok(join_response) = self.join_receiver.try_recv() {
            self.player_idx = Some(join_response.player_id);
            self.state.players.insert(
                join_response.player_id,
                Player::new(join_response.player_id),
            );
        }
    }
    pub fn jump(&mut self) {
        if let Some(player_idx) = self.player_idx {
            // Optimistic update
            self.state
                .players
                .get_mut(&player_idx)
                .unwrap()
                .last_jump_at = Some(chrono::Utc::now());

            // Send the jump action
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
        self.state.players.get_mut(&player_idx).unwrap().x += delta_x as f64;
    }
}

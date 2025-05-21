use std::sync::mpsc::{channel, Receiver};

use chrono::Utc;
use rust_socketio::{client::Client, ClientBuilder, Payload};

use crate::{
    event::{JoinResponse, PlayerAction},
    state::Player,
    Action, State, ACTION_CHANNEL, ERROR_CHANNEL, JOIN_CHANNEL, STATE_CHANNEL,
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
            state: State {
                players: vec![],
                timestamp: Utc::now(),
            },
            player_idx: None,
            client: ClientBuilder::new("https://f04e4c981694.ngrok.app")
                .on(ERROR_CHANNEL, |payload, _| {
                    eprintln!("Received error: {:?}", payload);
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
                .on(JOIN_CHANNEL, move |payload, _| {
                    println!("received join");
                    match payload {
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
        while let Ok(state) = self.state_receiver.try_recv() {
            self.state = state;
        }
    }
    fn join_update(&mut self) {
        while let Ok(join_response) = self.join_receiver.try_recv() {
            self.player_idx = Some(join_response.player_id);
            self.state
                .players
                .push(Player::new(join_response.player_id));
        }
    }
    pub fn jump(&mut self) {
        if let Some(player_idx) = self.player_idx {
            // Optimistic update
            self.state.players.get_mut(player_idx).unwrap().last_jump_at = Some(chrono::Utc::now());

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
        if let Some(player_idx) = self.player_idx {
            // Optimistic update
            self.state.players.get_mut(player_idx).unwrap().x += delta_x as f64;

            // Send the move action
            self.client
                .emit(
                    ACTION_CHANNEL,
                    Payload::Text(vec![serde_json::to_value(&Action::Player {
                        id: player_idx,
                        action: PlayerAction::Move {
                            delta_x: delta_x as _,
                        },
                    })
                    .unwrap()]),
                )
                .unwrap();
        }
    }
}

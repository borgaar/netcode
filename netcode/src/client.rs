use std::sync::mpsc::{channel, Receiver};

use rust_socketio::{client::Client, ClientBuilder, Payload};

use crate::{
    event::JoinResponse, state::Player, State, ACTION_CHANNEL, JOIN_CHANNEL, STATE_CHANNEL,
};

pub struct Game {
    state_receiver: Receiver<State>,
    join_receiver: Receiver<JoinResponse>,
    pub state: State,
    pub player_id: Option<usize>,
    client: Client,
}

impl Game {
    pub fn new() -> Self {
        let (state_sender, state_receiver) = channel::<State>();
        let (join_sender, join_receiver) = channel::<JoinResponse>();

        let game = Self {
            state_receiver,
            join_receiver,
            state: State::default(),
            player_id: None,
            client: ClientBuilder::new("http://0.0.0.0:7878")
                .on(STATE_CHANNEL, move |payload, _| match payload {
                    Payload::Text(text) => {
                        // let data = serde_json::from_str::<State>(text);
                        // state_sender.send(data).unwrap();
                    }
                    _ => {
                        eprintln!("Received bad payload on state");
                    }
                })
                .on(JOIN_CHANNEL, move |payload, _| match payload {
                    Payload::Text(text) => {
                        let data =
                            serde_json::from_value::<JoinResponse>(text.first().unwrap().clone())
                                .unwrap();
                        join_sender.send(data).unwrap();
                    }
                    _ => {
                        eprintln!("Received non-binary payload on join");
                    }
                })
                .connect()
                .unwrap(),
        };

        game
    }
    pub async fn join(&self) {
        self.client.emit(ACTION_CHANNEL, "").unwrap();
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
    pub fn get_own_player(&mut self) -> Option<&mut Player> {
        self.state
            .players
            .iter_mut()
            .find(|p| Some(p.id) == self.player_id)
    }
    fn join_update(&mut self) {
        while let Ok(join_response) = self.join_receiver.try_recv() {
            self.player_id = Some(join_response.player_id);
            self.state
                .players
                .push(Player::new(join_response.player_id));
        }
    }
    pub fn jump(&mut self) {
        if let Some(player) = self.get_own_player() {
            player.last_jump_at = Some(chrono::Utc::now());
        }
    }
}

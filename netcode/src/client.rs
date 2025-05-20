use rust_socketio::{client::Client, ClientBuilder, Payload, RawClient};

use crate::State;

pub struct NetcodeClient {
    socket: RawClient,
    pub state: State,
}

impl NetcodeClient {
    pub fn new(addr: [u8; 4], port: u16) -> Self {
        NetcodeClient {
            state: State {
                players: vec![Player {
                    id: 0,
                    x: 0.0,
                    last_jump_at: Utc::now(),
                }],
            },
            socket: ClientBuilder::new("http://localhost:7878")
                .namespace("/")
                .on("test", |payload, socket| {
                    match payload {
                        Payload::Text(str) => println!("Received: {:?}", str),
                        Payload::Binary(bin_data) => println!("Received bytes: {:#?}", bin_data),
                        _ => println!("Received unknown payload"),
                    }
                    socket
                        .emit("test", json!({"got ack": true}))
                        .expect("Server unreachable")
                })
                .on("error", |err, _| eprintln!("Error: {:#?}", err))
                .connect()
                .expect("Connection failed"),
        }
    }
    pub fn register(&mut self) {
        self.socket
            .emit("event", json!({"player_id": 0, "x": 0.0}))
            .expect("Server unreachable");
    }
    pub fn move_player() {}
}

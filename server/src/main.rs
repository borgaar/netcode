use axum::routing::get;
use netcode::{ACTION_CHANNEL, ERROR_CHANNEL, JOIN_CHANNEL, STATE_CHANNEL};
use serde_json::json;
use socketioxide::{
    extract::{AckSender, Data, SocketRef, State},
    SocketIo,
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

async fn on_connect(
    socket: SocketRef,
    Data(data): Data<serde_json::Value>,
    State(state): State<AppState>,
) {
    let state = state.state;

    let socket_state = state.clone();
    socket.on(
        ACTION_CHANNEL,
        async move |socket: SocketRef, Data::<serde_json::Value>(data)| {
            let event = serde_json::from_value::<netcode::Action>(data);

            let event = match event {
                Ok(e) => e,
                Err(err) => {
                    socket.emit(
                        ERROR_CHANNEL,
                        &format!("Error while parsing event payload: {}", err),
                    );
                    return;
                }
            };

            {
                match event.variant {
                    netcode::event::Variant::Jump(jump) => {
                        let player = &mut socket_state.lock().unwrap().players[event.player_id];
                        player.last_jump_at = jump.at;
                    }
                    netcode::event::Variant::Movement(movement) => {
                        let player = &mut socket_state.lock().unwrap().players[event.player_id];
                        player.x += movement
                    }
                    netcode::event::Variant::Join => {
                        let players = &mut socket_state.lock().unwrap().players;
                        let player_id = players.len();
                        players.push(netcode::Player::new(player_id));
                        socket.emit(JOIN_CHANNEL, &json!({ "player_id": player_id }));
                    }
                };
            }

            let response = serde_json::to_string(socket_state.as_ref()).unwrap();
            socket.emit(STATE_CHANNEL, &response).unwrap();
        },
    );

    let global_state = state.clone();
    tokio::spawn(async move {
        let state = &global_state;
        loop {
            {
                let state = state.lock().unwrap();
                let message = serde_json::to_string(&*state).unwrap();
                socket.emit(STATE_CHANNEL, &message);
            }

            tokio::time::sleep(Duration::from_millis(300)).await
        }
    });
}

#[derive(Debug, Clone, Default)]
struct AppState {
    state: Arc<Mutex<netcode::State>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState::default();

    let (layer, io) = SocketIo::builder().with_state(state).build_layer();
    io.ns("/", on_connect);

    let app = axum::Router::new().layer(layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7878").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

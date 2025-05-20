use axum::routing::get;
use netcode::{event::JoinResponse, ACTION_CHANNEL, ERROR_CHANNEL, JOIN_CHANNEL, STATE_CHANNEL};
use serde_json::json;
use socketioxide::{
    extract::{AckSender, Data, SocketRef, State},
    SocketIo,
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

const STATE_UPDATE_INTERVAL: Duration = Duration::from_millis(300);

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
                let mut state = socket_state.lock().unwrap();

                match event.variant {
                    netcode::event::Variant::Jump(jump) => {
                        state.player_jump(event.player_id, jump.at).unwrap();
                    }
                    netcode::event::Variant::Movement(movement) => {
                        state.player_move(event.player_id, movement);
                    }
                    netcode::event::Variant::Join => {
                        let player_id = state.player_join();
                        let response =
                            serde_json::to_string(&JoinResponse::new(player_id)).unwrap();
                        socket.local().emit(JOIN_CHANNEL, &response);
                    }
                };
            }
        },
    );

    let global_state = state.clone();
    tokio::spawn(async move {
        let state = &global_state;
        loop {
            {
                let state = state.lock().unwrap();
                let message = serde_json::to_string(&*state).unwrap();

                // Ignore error since we can just wait for the next state broadcast
                let _ = socket.emit(STATE_CHANNEL, &message);
            }

            tokio::time::sleep(STATE_UPDATE_INTERVAL).await
        }
    });
}

#[derive(Debug, Clone, Default)]
struct AppState {
    state: Arc<Mutex<netcode::State>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = AppState::default();

    let (layer, io) = SocketIo::builder().with_state(state).build_layer();
    io.ns("/", on_connect);

    let app = axum::Router::new().layer(layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7878").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

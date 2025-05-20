use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::routing::get;
use socketioxide::{
    extract::{AckSender, Data, SocketRef, State},
    SocketIo,
};

async fn on_connect(
    socket: SocketRef,
    Data(data): Data<serde_json::Value>,
    State(state): State<AppState>,
) {
    let state = state.state;

    let socket_state = state.clone();
    socket.on(
        "event",
        async move |socket: SocketRef, Data::<serde_json::Value>(data)| {
            let event = serde_json::from_value::<netcode::Event>(data);

            let event = match event {
                Ok(e) => e,
                Err(err) => {
                    socket.emit(
                        "error",
                        &format!("Error while parsing event payload: {}", err),
                    );
                    return;
                }
            };

            {
                let player = &mut socket_state.lock().unwrap().players[event.player_id];

                match event.variant {
                    netcode::event::Variant::Jump(jump) => {
                        player.last_jump_at = jump.at;
                    }
                    netcode::event::Variant::Movement(movement) => player.x += movement,
                };
            }

            let response = serde_json::to_string(socket_state.as_ref()).unwrap();
            socket.emit("state", &response).unwrap();
        },
    );

    let global_state = state.clone();
    tokio::spawn(async move {
        let state = &global_state;
        loop {
            {
                let state = state.lock().unwrap();
                let message = serde_json::to_string(&*state).unwrap();
                socket.emit("state", &message);
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

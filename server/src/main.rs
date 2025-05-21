use netcode::{event::JoinResponse, ACTION_CHANNEL, ERROR_CHANNEL, JOIN_CHANNEL, STATE_CHANNEL};
use socketioxide::{
    extract::{Data, SocketRef, State},
    SocketIo,
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

const STATE_UPDATE_INTERVAL: Duration = Duration::from_millis(300);

async fn on_connect(socket: SocketRef, State(state): State<AppState>) {
    let state = state.state;

    let socket_state = state.clone();
    socket.on(
        ACTION_CHANNEL,
        async move |socket: SocketRef, Data::<serde_json::Value>(data)| {
            let event = serde_json::from_value::<netcode::Action>(data);

            let event = match event {
                Ok(e) => e,
                Err(err) => {
                    socket
                        .emit(
                            ERROR_CHANNEL,
                            &format!("Error while parsing event payload: {}", err),
                        )
                        .unwrap();
                    return;
                }
            };

            {
                let mut state = socket_state.lock().unwrap();

                match event {
                    netcode::Action::Join => {
                        let player_id = state.player_join();
                        let response =
                            serde_json::to_string(&JoinResponse::new(player_id)).unwrap();
                        tokio::spawn(async move {
                            let _ = socket.local().emit(JOIN_CHANNEL, &response).await;
                        });
                    }
                    netcode::Action::Player { id, action } => match action {
                        netcode::event::PlayerAction::Leave => {
                            try_action(state.player_leave(id), socket);
                        }
                        netcode::event::PlayerAction::Jump { at } => {
                            try_action(state.player_jump(id, at), socket);
                        }
                        netcode::event::PlayerAction::Move { delta_x } => {
                            try_action(state.player_move(id, delta_x), socket);
                        }
                    },
                }
            }
        },
    );

    let global_state = state.clone();
    tokio::spawn(async move {
        let state = &global_state;
        loop {
            {
                let mut state = state.lock().unwrap();
                state.tick();
                let message = serde_json::to_string(&*state).unwrap();

                // Ignore error since we can just wait for the next state broadcast
                let _ = socket.emit(STATE_CHANNEL, &message);
            }

            tokio::time::sleep(STATE_UPDATE_INTERVAL).await
        }
    });
}

fn try_action(result: Result<(), netcode::state::StateError>, socket: SocketRef) {
    if let Err(e) = result {
        tokio::spawn(async move {
            let _ = socket.local().emit(ERROR_CHANNEL, &e.to_string()).await;
        });
    }
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

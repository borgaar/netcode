use netcode::{event::JoinResponse, ACTION_CHANNEL, ERROR_CHANNEL, JOIN_CHANNEL, STATE_CHANNEL};
use socketioxide::{
    extract::{Data, SocketRef, State},
    socket::DisconnectReason,
    SocketIo,
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

/// Time between each tick update on the server's state
const STATE_UPDATE_INTERVAL: Duration = Duration::from_millis(500);

/// Handles socket connections
async fn on_connect(socket: SocketRef, State(state): State<AppState>) {
    let state = state.state;
    let user_id = Arc::new(Mutex::new(0));

    let socket_state = state.clone();

    let socket_user_id = user_id.clone();
    socket.on(
        ACTION_CHANNEL,
        async move |socket: SocketRef, Data::<serde_json::Value>(data)| {
            let event = serde_json::from_value::<netcode::Action>(data);

            let event = match event {
                Ok(e) => e,
                Err(err) => {
                    let _ = socket.emit(
                        ERROR_CHANNEL,
                        &format!("Error while parsing event payload: {}", err),
                    );
                    return;
                }
            };

            {
                let mut state = socket_state.lock().unwrap();

                match event {
                    netcode::Action::Join => {
                        let player_id = state.player_join();
                        {
                            *socket_user_id.lock().unwrap() = player_id;
                        }
                        let response =
                            serde_json::to_string(&JoinResponse::new(player_id)).unwrap();
                        println!("Player joined the game. Got ID {player_id}");
                        tokio::spawn(socket.local().emit(JOIN_CHANNEL, &response));
                    }
                    netcode::Action::Player {
                        id: player_id,
                        action,
                    } => match action {
                        netcode::event::PlayerAction::Jump { at } => {
                            println!("Player {player_id} jumped at {at}");
                            try_action(state.player_jump(player_id, at), socket);
                        }
                        netcode::event::PlayerAction::Move { delta_x, id } => {
                            println!("Player {player_id} moved by {delta_x} units");
                            try_action(state.player_move(player_id, delta_x, id), socket);
                        }
                    },
                }
            }
        },
    );

    let disconnect_state = state.clone();
    socket.on_disconnect(async move |socket: SocketRef, _: DisconnectReason| {
        let user_id = *user_id.lock().unwrap();
        println!("Player {user_id} left the session");
        let mut state = disconnect_state.lock().unwrap();
        try_action(state.player_leave(user_id), socket);
    });

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
                // state.clear_ack();
            }

            tokio::time::sleep(STATE_UPDATE_INTERVAL).await
        }
    });
}

fn try_action(result: Result<(), netcode::state::StateError>, socket: SocketRef) {
    if let Err(e) = result {
        tokio::spawn(socket.local().emit(ERROR_CHANNEL, &e.to_string()));
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

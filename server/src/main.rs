//! Main entrypoint for the server-side SocketIO API.

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
const STATE_UPDATE_INTERVAL: Duration = Duration::from_millis(333);

/// Handles incoming socket connections from clients
async fn on_connect(socket: SocketRef, State(state): State<Arc<AppState>>) {
    let state = &state.state;
    let user_id = Arc::new(Mutex::new(0));

    let socket_state = state.clone();

    println!("new client connected");

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
                println!("Acquiring lock");
                let mut state = socket_state.lock().unwrap();
                println!("Lock acquired");

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
}

/// Tries applying a state action; sends a message to the error channel if it fails, without blocking the thread.
fn try_action(result: Result<(), netcode::state::StateError>, socket: SocketRef) {
    if let Err(e) = result {
        tokio::spawn(socket.local().emit(ERROR_CHANNEL, &e.to_string()));
    }
}

/// Global game state that can be cloned into multiple handles across threads.
#[derive(Debug, Clone, Default)]
struct AppState {
    state: Arc<Mutex<netcode::State>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = Arc::new(AppState::default());

    let (layer, io) = SocketIo::builder().with_state(state.clone()).build_layer();
    io.ns("/", on_connect);

    start_periodic_broadcast_to_namespace(io.clone(), state.clone());

    println!("Creating router");

    let app = axum::Router::new().layer(layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7878").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

// Starts broadcasting the state periodically to all clients to synchronize the game state.
fn start_periodic_broadcast_to_namespace(io: SocketIo, state: Arc<AppState>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(STATE_UPDATE_INTERVAL);

        loop {
            interval.tick().await;

            let message = state.state.lock().unwrap().tick();

            if let Err(e) = io.broadcast().emit(STATE_CHANNEL, &message).await {
                eprintln!("Failed to broadcast to namespace /: {}", e);
            }
        }
    });
}

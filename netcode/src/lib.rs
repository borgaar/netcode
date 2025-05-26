//! Netcode handling on server and client to synchronize and minimize lag for multiplayer gaming.

pub mod client;
pub mod event;
pub mod state;

pub use event::Action;
pub use state::State;

/// SocketIO channel name for client-side actions
pub const ACTION_CHANNEL: &str = "action";

/// SocketIO channel name for sending state updates to the client
pub const STATE_CHANNEL: &str = "state";

/// SocketIO channel name to send join information to the client
pub const JOIN_CHANNEL: &str = "join";

/// SocketIO channel name to send errors to the client
pub const ERROR_CHANNEL: &str = "error";

/// Max number of units traveled per second for a client. Is used to prevent cheating.
pub const MAX_UNITS_PER_SECOND: f64 = 2.5;

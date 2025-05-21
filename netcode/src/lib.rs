// pub mod client;
pub mod client;
pub mod event;
pub mod server;
pub mod state;

pub use event::Action;
pub use state::State;

pub const ACTION_CHANNEL: &str = "action";
pub const STATE_CHANNEL: &str = "state";
pub const JOIN_CHANNEL: &str = "join";
pub const ERROR_CHANNEL: &str = "error";

pub const MAX_UNITS_PER_MS: f64 = 2.5 / 1000.0;
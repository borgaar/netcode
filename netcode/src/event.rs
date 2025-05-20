use std::time;

use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Event {
    Jump(JumpEvent),
    Movement(MovementEvent)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JumpEvent {
    at: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MovementEvent {
    direction: Direction,
    amount: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Direction {
    Left,
    Right,
}

use std::time;

use chrono::Utc;

pub struct JumpEvent {
    at: chrono::DateTime<Utc>,
}

pub struct MovementEvent {
    direction: Direction,
    amount: f64,
}

pub enum Direction {
    Left,
    Right,
}

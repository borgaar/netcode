use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Action {
    pub player_id: usize,
    pub variant: Variant,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Variant {
    Join,
    Jump(JumpAction),
    Movement(f64),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JumpAction {
    pub at: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JoinResponse {
    player_id: usize,
}

impl JoinResponse {
    pub fn new(player_id: usize) -> Self {
        Self { player_id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn serialization() {
        let payload = Action { player_id: 1, variant: Variant::Movement(3.4) };
        
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
        panic!()
    }
}
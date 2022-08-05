use serde::{Serialize, Deserialize};

use crate::logic::Direction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Signal {
    Disconnect,
    Connect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SnakeEventType {
    Movement(Direction),
    Signal(Signal),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnakeEvent {
    pub event_type: SnakeEventType,
    pub event_owner: String,
}

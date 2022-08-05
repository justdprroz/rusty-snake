use crate::logic::Direction;

pub enum Signal {
    Disconnect,
    Connect
}

pub enum SnakeEventType {
    Movement(Direction),
    Signal(Signal),
}

pub struct SnakeEvent {
    pub event_type: SnakeEventType,
    pub event_owner: String
}
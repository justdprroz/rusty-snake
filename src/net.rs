use crate::logic::Direction;

#[derive(Debug)]
pub enum Signal {
    Disconnect,
}

#[derive(Debug)]
pub enum SnakeEvent {
    Movement(Direction),
    Signal(Signal)
}
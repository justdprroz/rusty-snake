#[derive(Debug)]
pub enum Direction {
    Up,
    Left,
    Down,
    Rigth,
}

#[derive(Debug)]
pub enum Signal {
    Disconnect,
}

#[derive(Debug)]
pub enum Input {
    Movement(Direction),
    Signal(Signal),
    Nothing,
}

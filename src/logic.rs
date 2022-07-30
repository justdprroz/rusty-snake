use rand::Rng;

use crate::net::SnakeEvent;

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
    Stop,
}

#[derive(Clone, Debug)]
pub struct Snake {
    pub body: Vec<(isize, isize)>,
    pub head: (isize, isize),
    pub direction: Direction,
    pub moved_from: Direction,
    pub name: String,
}

impl Snake {
    pub fn new(x: isize, y: isize, name: String) -> Self {
        Self {
            body: Vec::from([(x, y)]),
            head: (x, y),
            direction: Direction::Stop,
            moved_from: Direction::Stop,
            name: name,
        }
    }
}

#[derive(Debug)]
pub struct Game {
    snakes: Vec<Snake>,
    food: Vec<(isize, isize)>,
    size: (usize, usize),
    food_amount: usize,
    players: usize,
}

impl Game {
    pub fn new(width: usize, height: usize, food_amount: usize, players: usize) -> Self {
        Self {
            snakes: Vec::new(),
            food: {
                let mut poses = Vec::new();
                let mut rng = rand::thread_rng();
                for _ in 0..food_amount {
                    poses.push((rng.gen_range(0..width as isize), rng.gen_range(0..height as isize)));
                }
                poses
            },
            size: (width, height),
            food_amount: food_amount,
            players: players,
        }
    }
    pub fn add_player(&mut self, name: String) {
        let mut rng = rand::thread_rng();
        let pos = (rng.gen_range(0..self.size.0 as isize), rng.gen_range(0..self.size.1 as isize));
        self.snakes.push(Snake::new(pos.0, pos.1, name));
    }
    pub fn remove_player(&mut self, name: String) {
        self.snakes.retain(|snake| snake.name != name);
    }
    pub fn step(&mut self) {
        for snake in &mut self.snakes {
            let mut last = snake.head;
            match snake.direction {
                Direction::Up => {
                    last.1 -= 1;
                    snake.moved_from = Direction::Down;
                }
                Direction::Left => {
                    last.0 -= 1;
                    snake.moved_from = Direction::Right;
                }
                Direction::Down => {
                    last.1 += 1;
                    snake.moved_from = Direction::Up;
                }
                Direction::Right => {
                    last.0 += 1;
                    snake.moved_from = Direction::Left;
                }
                Direction::Stop => {}
            }
            if last != snake.head {
                snake.head = last;
                for cell in &mut snake.body {
                    let tmp = *cell;
                    *cell = last;
                    last = tmp;
                }
            }
            for food in &mut self.food {
                if *food == snake.head {
                    let mut rng = rand::thread_rng();
                    snake.body.push(last);
                    *food = (rng.gen_range(0..self.size.0 as isize), rng.gen_range(0..self.size.1 as isize));
                }
            }
        }
    }
    pub fn handle_events(&mut self, event: SnakeEvent, name: String) {
        // Find current snake by name
        let mut current_snake: Option<&mut Snake> = None;
        for snake in &mut self.snakes {
            if snake.name == name {
                current_snake = Some(snake);
            }
        }
        if let None = current_snake {
            return;
        }
        let current_snake = current_snake.unwrap();

        // process events
        match event {
            SnakeEvent::Movement(dir) => {
                match dir {
                    Direction::Up => {
                        if Direction::Up != current_snake.moved_from {
                            current_snake.direction = dir
                        }
                    },
                    Direction::Left => {
                        if Direction::Left != current_snake.moved_from {
                            current_snake.direction = dir
                        }
                    },
                    Direction::Down => {
                        if Direction::Down != current_snake.moved_from {
                            current_snake.direction = dir
                        }
                    },
                    Direction::Right => {
                        if Direction::Right != current_snake.moved_from {
                            current_snake.direction = dir
                        }
                    },
                    Direction::Stop => current_snake.direction = dir,
                }
            }
            SnakeEvent::Signal(signal) => match signal {
                crate::net::Signal::Disconnect => {
                    self.remove_player(name);
                }
            },
        }
    }
    pub fn get_snake(&self, name: String) -> Snake {
        let mut current_snake: Option<&Snake> = None;
        for snake in &self.snakes {
            if snake.name == name {
                current_snake = Some(snake);
            }
        }
        if let None = current_snake {
            return Snake::new(0, 0, "".to_string());
        }
        let current_snake = current_snake.unwrap();
        current_snake.clone()
    }
    pub fn get_food(&self) -> &Vec<(isize, isize)> {
        &self.food
    }
}

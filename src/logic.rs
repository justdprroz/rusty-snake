use rand::Rng;
use serde::{Serialize, Deserialize};

use crate::net::{Signal, SnakeEventType};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
    Stop,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snake {
    pub name: String,
    pub id: usize,
    pub body: Vec<(isize, isize)>,
    pub head: (isize, isize),
    pub direction: Direction,
    pub moved_from: Direction,
    pub alive: bool,
}

impl Snake {
    pub fn new(x: isize, y: isize, name: String, id: usize) -> Self {
        Self {
            name: name,
            id: id,
            body: Vec::from([(x, y)]),
            head: (x, y),
            direction: Direction::Stop,
            moved_from: Direction::Stop,
            alive: true,
        }
    }

    pub fn get_body(&self) -> &Vec<(isize, isize)> {
        &self.body
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Cell {
    Void,
    Wall,
    Player(usize),
    Food,
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellManager {
    size: (usize, usize),
    data: Vec<Vec<Cell>>,
}

impl CellManager {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            size: (width, height),
            data: (vec![vec![Cell::Empty; height]; width]),
        }
    }

    pub fn get_cell(&self, x: isize, y: isize) -> Cell {
        if x >= 0 && y >= 0 {
            let x = x as usize;
            let y = y as usize;
            if x < self.size.0 && y < self.size.1 {
                self.data[x][y].clone()
            } else {
                Cell::Void
            }
        } else {
            Cell::Void
        }
    }

    pub fn set_cell(&mut self, x: isize, y: isize, c: Cell) {
        if x >= 0 && y >= 0 {
            let x = x as usize;
            let y = y as usize;
            if x < self.size.0 && y < self.size.1 {
                self.data[x][y] = c;
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    // Settings
    pub size: (usize, usize),
    pub food_amount: usize,
    pub max_players: usize,
    pub teleport: bool,
    // Data
    pub snakes: Vec<Snake>,
    pub food: Vec<(isize, isize)>,
    pub cells: CellManager,
}

impl Game {
    pub fn new(
        width: usize,
        height: usize,
        food_amount: usize,
        players: usize,
        teleport: bool,
    ) -> Self {
        Self {
            size: (width, height),
            food_amount: food_amount,
            max_players: players,
            teleport: teleport,
            snakes: Vec::new(),
            food: Vec::new(),
            cells: CellManager::new(width, height),
        }
    }

    pub fn add_missing_food(&mut self) {
        let mut rng = rand::thread_rng();
        let missing_food = self.food_amount - self.food.len();
        'adding: for _ in 0..missing_food {
            let mut c = 0;
            self.food.push(loop {
                if c >= 50 {
                    break 'adding;
                }
                c += 1;
                let pos = (
                    rng.gen_range(0..self.size.0 as isize),
                    rng.gen_range(0..self.size.1 as isize),
                );
                if let Cell::Empty = self.cells.get_cell(pos.0, pos.1) {
                    self.cells.set_cell(pos.0, pos.1, Cell::Food);
                    break pos;
                }
            });
        }
    }

    pub fn add_player(&mut self, name: String) -> bool {
        if self.snakes.len() >= self.max_players {
            return false;
        }
        if let Some(_) = self.snakes.iter().position(|s| *s.name == name) {
            return false;
        }
        let mut rng = rand::thread_rng();
        let pos = (
            rng.gen_range(0..self.size.0 as isize),
            rng.gen_range(0..self.size.1 as isize),
        );
        let id = self.snakes.len();
        self.snakes.push(Snake::new(pos.0, pos.1, name, id));
        self.cells.set_cell(pos.0, pos.1, Cell::Player(id));
        true
    }

    pub fn remove_player(&mut self, name: String) {
        let snake_index = match self.snakes.iter().position(|x| x.name == name) {
            Some(index) => index,
            None => return,
        };
        let current_snake = self.snakes.remove(snake_index);
        for part in current_snake.body {
            self.cells.set_cell(part.0, part.1, Cell::Empty);
        }
    }

    pub fn step(&mut self) {
        for snake in &mut self.snakes {
            // Process Movement
            let mut next_pos = snake.head;
            match snake.direction {
                Direction::Up => {
                    next_pos.1 -= 1;
                }
                Direction::Left => {
                    next_pos.0 -= 1;
                }
                Direction::Down => {
                    next_pos.1 += 1;
                }
                Direction::Right => {
                    next_pos.0 += 1;
                }
                Direction::Stop => {}
            }

            if next_pos != snake.head {
                // If there any movement process logic
                let mut next_cell = self.cells.get_cell(next_pos.0, next_pos.1);
                if let Cell::Void = next_cell {
                    if self.teleport {
                        next_pos.0 = (next_pos.0 + self.size.0 as isize) % self.size.0 as isize;
                        next_pos.1 = (next_pos.1 + self.size.1 as isize) % self.size.1 as isize;
                        next_cell = self.cells.get_cell(next_pos.0, next_pos.1);
                    }
                }
                match next_cell {
                    Cell::Void => {
                        // Already checked and impossible to occur but panic
                        panic!("What The Fuck is going on here? Position: {:?}", next_pos);
                    }
                    Cell::Wall => {
                        unimplemented!("Oh man we don't have walls!")
                    }
                    Cell::Player(_) => {
                        // self.remove_player(snake.name);
                        snake.alive = false;
                        for part in &snake.body {
                            self.cells.set_cell(part.0, part.1, Cell::Empty);
                        }
                    }
                    Cell::Empty | Cell::Food => {
                        snake.head = next_pos;
                        self.cells
                            .set_cell(next_pos.0, next_pos.1, Cell::Player(snake.id));
                        for cell in &mut snake.body {
                            let tmp = *cell;
                            *cell = next_pos;
                            self.cells
                                .set_cell(next_pos.0, next_pos.1, Cell::Player(snake.id));
                            next_pos = tmp;
                        }
                        self.cells.set_cell(next_pos.0, next_pos.1, Cell::Empty);
                        if let Cell::Food = next_cell {
                            snake.body.push(next_pos);
                            self.cells
                                .set_cell(next_pos.0, next_pos.1, Cell::Player(snake.id));
                            self.food.retain(|f| *f != snake.head);
                        }
                        match snake.direction {
                            Direction::Up => {
                                snake.moved_from = Direction::Down;
                            }
                            Direction::Left => {
                                snake.moved_from = Direction::Right;
                            }
                            Direction::Down => {
                                snake.moved_from = Direction::Up;
                            }
                            Direction::Right => {
                                snake.moved_from = Direction::Left;
                            }
                            Direction::Stop => {}
                        }
                    }
                }
            }
        }
        self.snakes.retain(|s| s.alive == true);
    }
    pub fn handle_events(&mut self, event: SnakeEventType, name: String) {
        // Find current snake by name

        // process events
        match event {
            SnakeEventType::Movement(dir) => {
                let current_snake = match self.get_snake_mut(name.clone()) {
                    Some(snake) => snake,
                    None => return,
                };
                match dir {
                    Direction::Up => {
                        if Direction::Up != current_snake.moved_from {
                            current_snake.direction = dir
                        }
                    }
                    Direction::Left => {
                        if Direction::Left != current_snake.moved_from {
                            current_snake.direction = dir
                        }
                    }
                    Direction::Down => {
                        if Direction::Down != current_snake.moved_from {
                            current_snake.direction = dir
                        }
                    }
                    Direction::Right => {
                        if Direction::Right != current_snake.moved_from {
                            current_snake.direction = dir
                        }
                    }
                    Direction::Stop => current_snake.direction = dir,
                }
            }
            SnakeEventType::Signal(signal) => match signal {
                Signal::Disconnect => {
                    self.remove_player(name);
                }
                Signal::Connect => {
                    self.add_player(name);
                }
            },
        }
    }

    fn get_snake_mut(&mut self, name: String) -> Option<&mut Snake> {
        let mut current_snake: Option<&mut Snake> = None;
        for snake in &mut self.snakes {
            if snake.name == name {
                current_snake = Some(snake);
            }
        }
        current_snake
    }

    pub fn get_snake(&self, name: String) -> Option<&Snake> {
        let mut current_snake: Option<&Snake> = None;
        for snake in &self.snakes {
            if snake.name == name {
                current_snake = Some(snake);
            }
        }
        current_snake
    }

    pub fn get_food(&self) -> &Vec<(isize, isize)> {
        &self.food
    }

    pub fn get_owners_tables(&self) -> &CellManager {
        &self.cells
    }
}

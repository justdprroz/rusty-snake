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
    name: String,
    id: usize,
    body: Vec<(isize, isize)>,
    head: (isize, isize),
    direction: Direction,
    moved_from: Direction,
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
        }
    }

    pub fn get_body(&self) -> &Vec<(isize, isize)> {
        &self.body
    }
}

#[derive(Debug, Clone)]
pub enum Cell {
    Void,
    Wall,
    Player(usize),
    Food,
    Empty,
}

#[derive(Debug)]
pub struct CellManager {
    size: (usize, usize),
    data: Vec<Vec<Cell>>,
}

impl CellManager {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            size: (width, height),
            data: (vec![vec![Cell::Empty; width]; height]),
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

#[derive(Debug)]
pub struct Game {
    // Settings
    size: (usize, usize),
    food_amount: usize,
    players: usize,
    teleport: bool,
    // Data
    snakes: Vec<Snake>,
    food: Vec<(isize, isize)>,
    cells: CellManager,
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
            players: players,
            teleport: teleport,
            snakes: Vec::new(),
            food: Vec::new(),
            cells: CellManager::new(width, height),
        }
    }

    pub fn add_missing_food(&mut self) {
        let mut rng = rand::thread_rng();
        let missing_food = self.food_amount - self.food.len();
        for _ in 0..missing_food {
            self.food.push(loop {
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

    pub fn add_player(&mut self, name: String) {
        let mut rng = rand::thread_rng();
        let pos = (
            rng.gen_range(0..self.size.0 as isize),
            rng.gen_range(0..self.size.1 as isize),
        );
        let id = self.snakes.len();
        self.snakes.push(Snake::new(pos.0, pos.1, name, id));
        self.cells.set_cell(pos.0, pos.1, Cell::Player(id));
    }

    pub fn remove_player(&mut self, name: String) {
        let snake_index = self.snakes.iter().position(|x| x.name == name).unwrap();
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
                    snake.moved_from = Direction::Down;
                }
                Direction::Left => {
                    next_pos.0 -= 1;
                    snake.moved_from = Direction::Right;
                }
                Direction::Down => {
                    next_pos.1 += 1;
                    snake.moved_from = Direction::Up;
                }
                Direction::Right => {
                    next_pos.0 += 1;
                    snake.moved_from = Direction::Left;
                }
                Direction::Stop => {}
            }

            if next_pos != snake.head {
                // If there any movement process logic
                let mut next_cell = self.cells.get_cell(next_pos.0, next_pos.1);
                if let Cell::Void = next_cell {
                    if self.teleport {
                        next_pos.0 = (next_pos.0 + self.size.0 as isize) % self.size.1 as isize;
                        next_pos.1 = (next_pos.1 + self.size.1 as isize) % self.size.1 as isize;
                        next_cell = self.cells.get_cell(next_pos.0, next_pos.1);
                    }
                }
                match next_cell {
                    Cell::Void => {
                        panic!("{:?}", next_pos);
                    }
                    Cell::Wall => {}
                    Cell::Player(_) => {}
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
                            let food_index =
                                self.food.iter().position(|f| *f == snake.head).unwrap();
                            let mut rng = rand::thread_rng();
                            loop {
                                let pos = (
                                    rng.gen_range(0..self.size.0 as isize),
                                    rng.gen_range(0..self.size.1 as isize),
                                );
                                if let Cell::Empty = self.cells.get_cell(pos.0, pos.1) {
                                    *self.food.get_mut(food_index).unwrap() = pos;
                                    self.cells.set_cell(pos.0, pos.1, Cell::Food);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn handle_events(&mut self, event: SnakeEvent, name: String) {
        // Find current snake by name
        let current_snake = match self.get_snake_mut(name.clone()) {
            Some(snake) => snake,
            None => return,
        };
        // process events
        match event {
            SnakeEvent::Movement(dir) => match dir {
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
            },
            SnakeEvent::Signal(signal) => match signal {
                crate::net::Signal::Disconnect => {
                    self.remove_player(name);
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

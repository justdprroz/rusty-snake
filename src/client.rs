#![allow(dead_code)]

// Std Stuff
use std::io::{stdout, BufWriter, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, SystemTime};

use logic::{Direction, Game};
use net::SnakeEvent;

// Tokio Stuff
use tokio::sync::mpsc;

// Crossterm
use crossterm::style::{Color, Colors};
use crossterm::terminal::SetTitle;
use crossterm::{event, terminal, ExecutableCommand, QueueableCommand};

// Own Modules
mod logic;
mod net;
mod render;

// Renderings
use render::RenderBuffer;
use render::{RectangleShape, RenderChar};

// Global status
static APP_RUNNING: AtomicBool = AtomicBool::new(true);

// Clear terminal by escape sequence to position 0:0
fn clear_term() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

#[tokio::main]
async fn main() {
    terminal::enable_raw_mode().unwrap();
    let mut stdout = BufWriter::new(stdout());
    stdout.execute(event::EnableMouseCapture).unwrap();

    let (tx, mut rx) = mpsc::channel::<event::Event>(32);

    // spawn user input system
    tokio::spawn(async move {
        while APP_RUNNING.load(Ordering::Relaxed) {
            if event::poll(Duration::from_millis(100)).unwrap() {
                tx.send(event::read().unwrap()).await.unwrap();
            };
        }
    });

    let (width, height) = (
        terminal::size().unwrap().0 as usize,
        terminal::size().unwrap().1 as usize,
    );
    let mut buffer = RenderBuffer::new(width, height);

    // Game Stuff
    let (gw, gh) = (30, 30);
    let mut game: Game = Game::new(gw, gh, 5, 5);
    game.add_player("just".to_string());

    let mut time = SystemTime::now();

    let mut use_fancy: bool = true;
    let mut use_unicode: bool = true;

    // Game Loop
    while APP_RUNNING.load(Ordering::Relaxed) {
        stdout
            .queue(SetTitle((time.elapsed().unwrap().as_micros()) as u64))
            .unwrap();
        time = SystemTime::now();
        // get_input
        loop {
            if let Ok(input) = rx.try_recv() {
                match input {
                    event::Event::Key(event) => match event.code {
                        event::KeyCode::Char(c) => match c {
                            'w' => game.handle_events(
                                SnakeEvent::Movement(Direction::Up),
                                "just".to_string(),
                            ),
                            'a' => game.handle_events(
                                SnakeEvent::Movement(Direction::Left),
                                "just".to_string(),
                            ),
                            's' => game.handle_events(
                                SnakeEvent::Movement(Direction::Down),
                                "just".to_string(),
                            ),
                            'd' => game.handle_events(
                                SnakeEvent::Movement(Direction::Right),
                                "just".to_string(),
                            ),
                            'u' => use_unicode = !use_unicode,
                            'f' => use_fancy = !use_fancy,
                            _ => {}
                        },
                        event::KeyCode::Esc => APP_RUNNING.store(false, Ordering::Relaxed),
                        event::KeyCode::Enter => game.handle_events(
                            SnakeEvent::Movement(Direction::Stop),
                            "just".to_string(),
                        ),
                        _ => {}
                    },
                    event::Event::Resize(w, h) => buffer.resize(w as usize, h as usize),
                    event::Event::Mouse(event) => match event.kind {
                        event::MouseEventKind::Down(_) => {
                            stdout
                                .queue(SetTitle(format!("{}:{}", event.column, event.row)))
                                .unwrap();
                        }
                        _ => {}
                    },
                };
            } else {
                break;
            }
        }
        // do logic

        game.step();

        //draw
        buffer.clear(RenderChar::empty());

        if use_unicode && use_fancy {
            let r = RectangleShape::new(
                0,
                0,
                1,
                1,
                RenderChar::new(
                    '┏',
                    Colors {
                        foreground: Some(Color::Blue),
                        background: None,
                    },
                ),
                false,
            );
            buffer.draw(&r);

            let r = RectangleShape::new(
                (gw + 1) as isize,
                0,
                1,
                1,
                RenderChar::new(
                    '┓',
                    Colors {
                        foreground: Some(Color::Blue),
                        background: None,
                    },
                ),
                false,
            );
            buffer.draw(&r);

            let r = RectangleShape::new(
                0,
                (gh + 1) as isize,
                1,
                1,
                RenderChar::new(
                    '┗',
                    Colors {
                        foreground: Some(Color::Blue),
                        background: None,
                    },
                ),
                false,
            );
            buffer.draw(&r);

            let r = RectangleShape::new(
                (gw + 1) as isize,
                (gh + 1) as isize,
                1,
                1,
                RenderChar::new(
                    '┛',
                    Colors {
                        foreground: Some(Color::Blue),
                        background: None,
                    },
                ),
                false,
            );
            buffer.draw(&r);

            let r = RectangleShape::new(
                1,
                0,
                gw as isize,
                1,
                RenderChar::new(
                    '━',
                    Colors {
                        foreground: Some(Color::Blue),
                        background: None,
                    },
                ),
                false,
            );
            buffer.draw(&r);

            let r = RectangleShape::new(
                1,
                gh as isize + 1,
                gw as isize,
                1,
                RenderChar::new(
                    '━',
                    Colors {
                        foreground: Some(Color::Blue),
                        background: None,
                    },
                ),
                false,
            );
            buffer.draw(&r);

            let r = RectangleShape::new(
                0,
                1,
                1,
                gh as isize,
                RenderChar::new(
                    '┃',
                    Colors {
                        foreground: Some(Color::Blue),
                        background: None,
                    },
                ),
                false,
            );
            buffer.draw(&r);

            let r = RectangleShape::new(
                gw as isize + 1,
                1,
                1,
                gh as isize,
                RenderChar::new(
                    '┃',
                    Colors {
                        foreground: Some(Color::Blue),
                        background: None,
                    },
                ),
                false,
            );
            buffer.draw(&r);
        } else {
            let r = RectangleShape::new(
                0,
                0,
                (gw + 2) as isize,
                (gh + 2) as isize,
                RenderChar::new(
                    '#',
                    Colors {
                        foreground: Some(Color::Blue),
                        background: None,
                    },
                ),
                false,
            );
            buffer.draw(&r);
        }

        for food in game.get_food() {
            let r = RectangleShape::new(
                (food.0 + 1) as isize,
                (food.1 + 1) as isize,
                1,
                1,
                RenderChar::new(
                    if use_unicode && use_fancy { '※' } else { '*' },
                    Colors {
                        foreground: Some(Color::Red),
                        background: None,
                    },
                ),
                false,
            );
            buffer.draw(&r);
        }

        let my_snake = game.get_snake("just".to_string());
        for part_index in (1..my_snake.body.len()).rev() {
            let part = my_snake.body[part_index];
            let next_part = my_snake.body[part_index - 1];
            let diff_next = if part.0 - next_part.0 == 1 {
                4
            } else if part.0 - next_part.0 == -1 {
                2
            } else if part.1 - next_part.1 == 1 {
                1
            } else if part.1 - next_part.1 == -1 {
                3
            } else {
                5
            };

            let cur_char = if use_fancy {
                let char_set = if use_unicode {
                    ['│', '─', '.', '└', '┐', '┘', '┌']
                } else {
                    ['|', '-', '.', '\\', '\\', '/', '/']
                };
                if part_index == my_snake.body.len() - 1 {
                    if diff_next == 1 || diff_next == 3 {
                        char_set[0]
                    } else if diff_next == 2 || diff_next == 4 {
                        char_set[1]
                    } else {
                        char_set[2]
                    }
                } else {
                    let prev_part = my_snake.body[part_index + 1];
                    let diff_prev = if part.0 - prev_part.0 == 1 {
                        4
                    } else if part.0 - prev_part.0 == -1 {
                        2
                    } else if part.1 - prev_part.1 == 1 {
                        1
                    } else if part.1 - prev_part.1 == -1 {
                        3
                    } else {
                        5
                    };
                    if diff_next == 1 && diff_prev == 2 || diff_next == 2 && diff_prev == 1 {
                        char_set[3]
                    } else if diff_next == 3 && diff_prev == 4 || diff_next == 4 && diff_prev == 3 {
                        char_set[4]
                    } else if diff_next == 1 && diff_prev == 4 || diff_next == 4 && diff_prev == 1 {
                        char_set[5]
                    } else if diff_next == 3 && diff_prev == 2 || diff_next == 2 && diff_prev == 3 {
                        char_set[6]
                    } else if diff_next == 1 && diff_prev == 3 || diff_next == 3 && diff_prev == 1 {
                        char_set[0]
                    } else if diff_next == 2 && diff_prev == 4 || diff_next == 4 && diff_prev == 2 {
                        char_set[1]
                    } else {
                        char_set[2]
                    }
                }
            } else {
                '0'
            };

            let r = RectangleShape::new(
                (part.0 + 1) as isize,
                (part.1 + 1) as isize,
                1,
                1,
                RenderChar::new(
                    cur_char,
                    Colors {
                        foreground: Some(Color::Yellow),
                        background: None,
                    },
                ),
                false,
            );
            buffer.draw(&r);
        }

        let r = RectangleShape::new(
            (my_snake.head.0 + 1) as isize,
            (my_snake.head.1 + 1) as isize,
            1,
            1,
            RenderChar::new(
                '@',
                Colors {
                    foreground: Some(Color::Green),
                    background: None,
                },
            ),
            false,
        );
        buffer.draw(&r);

        clear_term();
        buffer.render_to(&mut stdout);
        stdout.flush().unwrap();

        thread::sleep(Duration::from_millis(1000 / 10));
    }
    stdout.execute(event::DisableMouseCapture).unwrap();
    terminal::disable_raw_mode().unwrap();
}

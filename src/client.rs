// #![allow(dead_code)]

// Std Stuff
use logic::{Cell, Direction, Game};
use net::{Signal, SnakeEvent, SnakeEventType};
use std::io::{stdout, BufWriter, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::{Duration, SystemTime};

// Crossterm
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::style::{Color, Colors};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, SetTitle};
use crossterm::{event, terminal, ExecutableCommand, QueueableCommand};

// Own Modules
mod logic;
mod net;

// Renderings
use rusty_ascii_graphics::RenderBuffer;
use rusty_ascii_graphics::{RectangleShape, RenderChar};

// Global status
static APP_RUNNING: AtomicBool = AtomicBool::new(true);

fn border_colors() -> Colors {
    Colors {
        foreground: Some(Color::Blue),
        background: None,
    }
}

fn server(tx: Sender<Game>, mut rx: Receiver<SnakeEvent>) {
    let (gw, gh) = (40, 20);
    let mut game: Game = Game::new(gw, gh, 4, 5, true);
    game.add_missing_food();
    loop {
        loop {
            match rx.try_recv() {
                Ok(event) => game.handle_events(event.event_type, event.event_owner),
                Err(_) => break,
            }
        }
        game.step();
        game.add_missing_food();
        tx.send(game.clone()).unwrap();
        thread::sleep(Duration::from_millis(100));
    }
}

fn client(mut rx: Receiver<Game>, tx: Sender<SnakeEvent>) {
    terminal::enable_raw_mode().unwrap();
    let mut stdout = BufWriter::new(stdout());
    stdout.execute(EnterAlternateScreen).unwrap();
    stdout.execute(Hide).unwrap();
    stdout.execute(event::EnableMouseCapture).unwrap();

    // spawn user input system
    // tokio::spawn(async move {
    //     while APP_RUNNING.load(Ordering::Relaxed) {
    //         if event::poll(Duration::from_millis(100)).unwrap() {
    //             tx.send(event::read().unwrap()).await.unwrap();
    //         };
    //     }
    // });

    let (width, height) = (
        terminal::size().unwrap().0 as usize,
        terminal::size().unwrap().1 as usize,
    );

    let mut buffer = RenderBuffer::new(width, height);

    // Game Stuff
    let mut game: Game = rx.recv().unwrap();
    tx.send(SnakeEvent {
        event_type: SnakeEventType::Signal(Signal::Connect),
        event_owner: "just".to_string(),
    })
    .unwrap();
    let (gw, gh) = (game.size.0, game.size.1);

    // let mut time = SystemTime::now();

    let mut use_fancy: bool = true;
    let mut use_unicode: bool = false;
    let mut use_debug: bool = false;
    let mut use_slow_mo: bool = false;
    let mut use_rgb: bool = true;

    // Game Loop
    'game_loop: while APP_RUNNING.load(Ordering::Relaxed) {
        // stdout
        //     .queue(SetTitle((time.elapsed().unwrap().as_micros()) as u64))
        //     .unwrap();
        // time = SystemTime::now();
        // get_input
        loop {
            if event::poll(Duration::from_millis(10)).unwrap() {
                match event::read().unwrap() {
                    event::Event::Key(event) => match event.code {
                        event::KeyCode::Char(c) => match c {
                            'w' => tx
                                .send(SnakeEvent {
                                    event_type: SnakeEventType::Movement(Direction::Up),
                                    event_owner: "just".to_string(),
                                })
                                .unwrap(),
                            'a' => tx
                                .send(SnakeEvent {
                                    event_type: SnakeEventType::Movement(Direction::Left),
                                    event_owner: "just".to_string(),
                                })
                                .unwrap(),
                            's' => tx
                                .send(SnakeEvent {
                                    event_type: SnakeEventType::Movement(Direction::Down),
                                    event_owner: "just".to_string(),
                                })
                                .unwrap(),
                            'd' => tx
                                .send(SnakeEvent {
                                    event_type: SnakeEventType::Movement(Direction::Right),
                                    event_owner: "just".to_string(),
                                })
                                .unwrap(),
                            // 'r' => {
                            //     tx.send(SnakeEvent {
                            //         event_type: SnakeEventType::Signal(Signal::Disconnect),
                            //         event_owner: "just".to_string(),
                            //     })
                            //     .unwrap();
                            //     tx.send(SnakeEvent {
                            //         event_type: SnakeEventType::Signal(Signal::Connect),
                            //         event_owner: "just".to_string(),
                            //     })
                            //     .unwrap();
                            // }
                            'u' => use_unicode = !use_unicode,
                            'c' => use_rgb = !use_rgb,
                            'f' => use_fancy = !use_fancy,
                            '\\' => use_debug = !use_debug,
                            '/' => use_slow_mo = !use_slow_mo,
                            _ => {}
                        },
                        event::KeyCode::Esc => {
                            tx.send(SnakeEvent {
                                event_type: SnakeEventType::Signal(Signal::Disconnect),
                                event_owner: "just".to_string(),
                            })
                            .unwrap();
                            APP_RUNNING.store(false, Ordering::Relaxed);
                            break 'game_loop;
                        }
                        event::KeyCode::Enter => tx
                            .send(SnakeEvent {
                                event_type: SnakeEventType::Movement(Direction::Stop),
                                event_owner: "just".to_string(),
                            })
                            .unwrap(),
                        _ => {}
                    },
                    event::Event::Resize(_, _) => buffer.resize(
                        terminal::size().unwrap().0 as usize,
                        terminal::size().unwrap().1 as usize,
                    ),
                    event::Event::Mouse(event) => match event.kind {
                        event::MouseEventKind::Down(_) => {
                            stdout
                                .execute(SetTitle(format!("{}:{}", event.column, event.row)))
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

        game = rx.recv().unwrap();

        //dra
        buffer.clear(RenderChar::empty());

        if use_fancy {
            let char_set = [
                RenderChar::new('╔', border_colors()),
                RenderChar::new('╗', border_colors()),
                RenderChar::new('╚', border_colors()),
                RenderChar::new('╝', border_colors()),
                RenderChar::new('═', border_colors()),
                RenderChar::new('║', border_colors()),
            ];
            let r = RectangleShape::new(0, 0, 1, 1, char_set[0].clone(), false);
            buffer.draw(&r);

            let r = RectangleShape::new((gw + 1) as isize, 0, 1, 1, char_set[1].clone(), false);
            buffer.draw(&r);

            let r = RectangleShape::new(0, (gh + 1) as isize, 1, 1, char_set[2].clone(), false);
            buffer.draw(&r);

            let r = RectangleShape::new(
                (gw + 1) as isize,
                (gh + 1) as isize,
                1,
                1,
                char_set[3].clone(),
                false,
            );
            buffer.draw(&r);

            let r = RectangleShape::new(1, 0, gw as isize, 1, char_set[4].clone(), false);
            buffer.draw(&r);

            let r = RectangleShape::new(
                1,
                gh as isize + 1,
                gw as isize,
                1,
                char_set[4].clone(),
                false,
            );
            buffer.draw(&r);

            let r = RectangleShape::new(0, 1, 1, gh as isize, char_set[5].clone(), false);
            buffer.draw(&r);

            let r = RectangleShape::new(
                gw as isize + 1,
                1,
                1,
                gh as isize,
                char_set[5].clone(),
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

        if let Some(my_snake) = game.get_snake("just".to_string()) {
            for part_index in (1..my_snake.get_body().len()).rev() {
                let part = my_snake.get_body()[part_index];
                let next_part = my_snake.get_body()[part_index - 1];
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
                    let char_set = if true {
                        ['│', '─', '.', '└', '┐', '┘', '┌']
                    } else {
                        ['|', '-', '.', '\\', '\\', '/', '/']
                    };
                    if part_index == my_snake.get_body().len() - 1 {
                        if diff_next == 1 || diff_next == 3 {
                            char_set[0]
                        } else if diff_next == 2 || diff_next == 4 {
                            char_set[1]
                        } else {
                            char_set[2]
                        }
                    } else {
                        let prev_part = my_snake.get_body()[part_index + 1];
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
                        } else if diff_next == 3 && diff_prev == 4
                            || diff_next == 4 && diff_prev == 3
                        {
                            char_set[4]
                        } else if diff_next == 1 && diff_prev == 4
                            || diff_next == 4 && diff_prev == 1
                        {
                            char_set[5]
                        } else if diff_next == 3 && diff_prev == 2
                            || diff_next == 2 && diff_prev == 3
                        {
                            char_set[6]
                        } else if diff_next == 1 && diff_prev == 3
                            || diff_next == 3 && diff_prev == 1
                        {
                            char_set[0]
                        } else if diff_next == 2 && diff_prev == 4
                            || diff_next == 4 && diff_prev == 2
                        {
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
                (my_snake.get_body()[0].0 + 1) as isize,
                (my_snake.get_body()[0].1 + 1) as isize,
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
        } else {
            tx.send(SnakeEvent {
                event_type: SnakeEventType::Signal(Signal::Connect),
                event_owner: "just".to_string(),
            })
            .unwrap();
        }

        let colors = if use_rgb {
            [
                Some(Color::Rgb { r: 127, g: 0, b: 0 }),
                Some(Color::Rgb {
                    r: 127,
                    g: 0,
                    b: 127,
                }),
                Some(Color::Rgb { r: 0, g: 0, b: 0 }),
                Some(Color::Rgb { r: 0, g: 127, b: 0 }),
            ]
        } else {
            [
                Some(Color::Red),
                Some(Color::Magenta),
                Some(Color::Black),
                Some(Color::DarkGreen),
            ]
        };

        if use_debug {
            for x in 0..gw + 2 {
                for y in 0..gh + 2 {
                    let cc = buffer.get((x) as isize, (y) as isize);
                    let o = game
                        .get_owners_tables()
                        .get_cell(x as isize - 1, y as isize - 1);
                    let nc = RenderChar::new(
                        cc.char,
                        match o {
                            Cell::Player(_) => Colors {
                                foreground: cc.colors.foreground,
                                background: colors[0],
                            },
                            Cell::Food => Colors {
                                foreground: cc.colors.foreground,
                                background: colors[1],
                            },
                            Cell::Wall | Cell::Void => Colors {
                                foreground: cc.colors.foreground,
                                background: colors[2],
                            },
                            Cell::Empty => Colors {
                                foreground: cc.colors.foreground,
                                background: colors[3],
                            },
                        },
                    );
                    buffer.put((x) as isize, (y) as isize, nc);
                }
            }
        }

        stdout.queue(MoveTo(0, 0)).unwrap();
        buffer.render_to(&mut stdout);

        stdout.flush().unwrap();
    }
    stdout.execute(LeaveAlternateScreen).unwrap();
    stdout.execute(event::DisableMouseCapture).unwrap();
    stdout.execute(Show).unwrap();
    terminal::disable_raw_mode().unwrap();
}

#[tokio::main]
async fn main() {
    let (tx_game, rx_game) = channel::<Game>();
    let (tx_event, rx_event) = channel::<SnakeEvent>();
    thread::Builder::new()
        .name("Server Thread".to_string())
        .spawn(move || server(tx_game, rx_event))
        .unwrap();
    let client_handle = thread::Builder::new()
        .name("Client Thread".to_string())
        .spawn(move || client(rx_game, tx_event))
        .unwrap();
    client_handle.join().unwrap();
}

// Std stuff
use std::io::{stdout, BufWriter, Write};
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::{fs, thread};

// Lazy static for runtime static variables
use lazy_static::lazy_static;

// Tokio
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::broadcast::{channel, Receiver, Sender};

// Crossterm
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::style::{Color, Colors};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, SetTitle};
use crossterm::{event, terminal, ExecutableCommand, QueueableCommand};

// Rusty Snake logic lib
use rusty_snake::{Cell, Direction, Game};
use rusty_snake::{Signal, SnakeEvent, SnakeEventType};

// Renderings
use rusty_ascii_graphics::RenderBuffer;
use rusty_ascii_graphics::{RectangleShape, RenderChar};

// Global variables
static APP_RUNNING: AtomicBool = AtomicBool::new(true);
lazy_static! {
    pub static ref CONF: toml::Value =
        toml::from_str(&fs::read_to_string("Client.toml").unwrap()).unwrap();
    pub static ref USERNAME: String = CONF["username"].as_str().unwrap().to_string();
    pub static ref SERVER_ADDRESS: String = CONF["server_address"].as_str().unwrap().to_string();
}

// Help functions
fn border_colors() -> Colors {
    Colors {
        foreground: Some(Color::Blue),
        background: None,
    }
}

// Function for integrated server thread
#[allow(dead_code)]
fn server(tx: Sender<Game>, mut rx: Receiver<SnakeEvent>) {
    let (gw, gh) = (40, 20);
    let mut game: Game = Game::new(gw, gh, 10, 5, true);
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

// Function for communicating with server
async fn socket(tx: Sender<Game>, mut rx: Receiver<SnakeEvent>, mut socket: TcpStream) {
    println!("Entering Socket Thread");
    while APP_RUNNING.load(Ordering::Relaxed) {
        let mut buf_len = [0u8; size_of::<usize>()];
        tokio::select! {
            result = socket.read_exact(&mut buf_len) => {
                let bytes = result.unwrap();
                if bytes == 0 {
                    println!("Exiting Socket Thread 1");
                    break;
                }
                let packet_len = bincode::deserialize(&buf_len).unwrap();
                let mut buf = vec![0u8; packet_len];
                socket.read_exact(&mut buf).await.unwrap();
                let game: Game = bincode::deserialize::<Game>(&buf).unwrap();
                tx.send(game).unwrap();
            },
            result = rx.recv() => {
                let event = result.unwrap();
                let buf = bincode::serialize(&event).unwrap();
                socket.write_all(&buf).await.unwrap();
            },
            else => {
                println!("Nothing to proceed");
            }
        }
    }
    loop {
        if let Ok(event) = rx.try_recv() {
            let buf = bincode::serialize(&event).unwrap();
            socket.write_all(&buf).await.unwrap();
        } else {
            break;
        }
    }
    println!("Exiting Socket Thread 2");
}

// Main rendering function
async fn client(mut rx: Receiver<Game>, tx: Sender<SnakeEvent>) {
    println!("Entered Render Thread");
    let mut stdout = BufWriter::new(stdout());
    terminal::enable_raw_mode().unwrap();
    stdout.execute(EnterAlternateScreen).unwrap();
    stdout.execute(Hide).unwrap();
    stdout.execute(event::EnableMouseCapture).unwrap();
    let (width, height) = (
        terminal::size().unwrap().0 as usize,
        terminal::size().unwrap().1 as usize,
    );

    let mut buffer = RenderBuffer::new(width, height);

    // Game Stuff
    println!("Waiting For Game Copy");
    // let mut game: Game = rx.recv().await.unwrap();
    let mut game = Game::new(40, 20, 0, 0, true);
    println!("Got Game Copy");
    tx.send(SnakeEvent {
        event_type: SnakeEventType::Signal(Signal::Connect),
        event_owner: USERNAME.to_string(),
    })
    .unwrap();
    println!("Send Connect");
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
        'events_or_game: loop {
            if let Ok(new_game) = rx.try_recv() {
                game = new_game;
                break 'events_or_game;
            } else {
                'events: loop {
                    if event::poll(Duration::from_millis(10)).unwrap() {
                        match event::read().unwrap() {
                            event::Event::Key(event) => match event.code {
                                event::KeyCode::Char(c) => match c {
                                    'w' => {
                                        tx.send(SnakeEvent {
                                            event_type: SnakeEventType::Movement(Direction::Up),
                                            event_owner: USERNAME.to_string(),
                                        })
                                        .unwrap();
                                    }
                                    'a' => {
                                        tx.send(SnakeEvent {
                                            event_type: SnakeEventType::Movement(Direction::Left),
                                            event_owner: USERNAME.to_string(),
                                        })
                                        .unwrap();
                                    }
                                    's' => {
                                        tx.send(SnakeEvent {
                                            event_type: SnakeEventType::Movement(Direction::Down),
                                            event_owner: USERNAME.to_string(),
                                        })
                                        .unwrap();
                                    }
                                    'd' => {
                                        tx.send(SnakeEvent {
                                            event_type: SnakeEventType::Movement(Direction::Right),
                                            event_owner: USERNAME.to_string(),
                                        })
                                        .unwrap();
                                    }
                                    'q' => {
                                        tx.send(SnakeEvent {
                                            event_type: SnakeEventType::Signal(Signal::Disconnect),
                                            event_owner: USERNAME.to_string().to_owned(),
                                        })
                                        .unwrap();
                                    }
                                    'r' => {
                                        tx.send(SnakeEvent {
                                            event_type: SnakeEventType::Signal(Signal::Connect),
                                            event_owner: USERNAME.to_string(),
                                        })
                                        .unwrap();
                                    }
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
                                        event_owner: USERNAME.to_string(),
                                    })
                                    .unwrap();
                                    // APP_RUNNING.store(false, Ordering::Relaxed);
                                    break 'game_loop;
                                }
                                event::KeyCode::Enter => {
                                    tx.send(SnakeEvent {
                                        event_type: SnakeEventType::Movement(Direction::Stop),
                                        event_owner: USERNAME.to_string(),
                                    })
                                    .unwrap();
                                }
                                _ => {}
                            },
                            event::Event::Resize(_, _) => buffer.resize(
                                terminal::size().unwrap().0 as usize,
                                terminal::size().unwrap().1 as usize,
                            ),
                            event::Event::Mouse(event) => match event.kind {
                                event::MouseEventKind::Down(_) => {
                                    stdout
                                        .execute(SetTitle(format!(
                                            "{}:{}",
                                            event.column, event.row
                                        )))
                                        .unwrap();
                                }
                                _ => {}
                            },
                        };
                    } else {
                        break 'events;
                    }
                }
            }
        }

        // do logic

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

        let mut my_snake_found: bool = false;

        for snake in &game.snakes {
            let (tail_color, head_color) = if snake.name == USERNAME.to_string() {
                my_snake_found = true;
                (Some(Color::Yellow), Some(Color::Green))
            } else {
                (Some(Color::Red), Some(Color::Blue))
            };
            for part_index in (1..snake.get_body().len()).rev() {
                let part = snake.get_body()[part_index];
                let next_part = snake.get_body()[part_index - 1];
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
                    if part_index == snake.get_body().len() - 1 {
                        if diff_next == 1 || diff_next == 3 {
                            char_set[0]
                        } else if diff_next == 2 || diff_next == 4 {
                            char_set[1]
                        } else {
                            char_set[2]
                        }
                    } else {
                        let prev_part = snake.get_body()[part_index + 1];
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
                            foreground: tail_color,
                            background: None,
                        },
                    ),
                    false,
                );
                buffer.draw(&r);
            }

            let r = RectangleShape::new(
                (snake.get_body()[0].0 + 1) as isize,
                (snake.get_body()[0].1 + 1) as isize,
                1,
                1,
                RenderChar::new(
                    '@',
                    Colors {
                        foreground: head_color,
                        background: None,
                    },
                ),
                false,
            );
            buffer.draw(&r);
        }
        if !my_snake_found {
            tx.send(SnakeEvent {
                event_type: SnakeEventType::Signal(Signal::Connect),
                event_owner: USERNAME.to_string(),
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
    println!("Exiting Render Thread");
}

#[tokio::main]
async fn main() {
    println!("{}", *SERVER_ADDRESS);
    println!("{}", USERNAME.to_string());

    let (tx_game, _rx_game) = channel::<Game>(32);
    let (tx_event, _rx_event) = channel::<SnakeEvent>(32);

    let _use_remote = true;
    let stream = TcpStream::connect(SERVER_ADDRESS.to_string())
        .await
        .unwrap();

    let socket_handle = tokio::spawn(socket(tx_game, _rx_event, stream));
    let client_handle = thread::Builder::new()
        .name("Client Thread".to_string())
        .spawn(move || client(_rx_game, tx_event))
        .unwrap();
    println!("Spawned Client thread");
    client_handle.join().unwrap().await;
    socket_handle.await.unwrap();
    println!("Exiting Main Thread");
}

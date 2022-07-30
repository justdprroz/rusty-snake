#![allow(dead_code)]

// Std Stuff
use std::io::{stdout, BufWriter};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, SystemTime};

use crossterm::style::{SetColors, SetForegroundColor, Color, Colors};
use logic::{Direction, Game};
use net::SnakeEvent;
// Tokio Stuff
use tokio::sync::mpsc;

// Crossterm
use crossterm::terminal::SetTitle;
use crossterm::{event, terminal, ExecutableCommand};

// Own Modules
mod logic;
mod net;
mod render;

// Renderings
use render::{RectangleShape, RenderChar};
use render::RenderBuffer;

// Global status
static APP_RUNNING: AtomicBool = AtomicBool::new(true);

// Clear terminal by escape sequence to position 0:0
fn clear_term() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

#[tokio::main]
async fn main() {
    let mut stdout = BufWriter::new(stdout());
    stdout.execute(event::EnableMouseCapture).unwrap();
    terminal::enable_raw_mode().unwrap();
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

    // Game Loop
    while APP_RUNNING.load(Ordering::Relaxed) {
        stdout
            .execute(SetTitle((time.elapsed().unwrap().as_micros()) as u64))
            .unwrap();
        time = SystemTime::now();
        // get_input
        loop {
            if let Ok(input) = rx.try_recv() {
                match input {
                    event::Event::Key(event) => match event.code {
                        event::KeyCode::Char(c) => match c {
                            'w' => game
                                .handle_events(SnakeEvent::Movement(Direction::Up), "just".to_string()),
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
                            _ => {}
                        },
                        event::KeyCode::Esc => APP_RUNNING.store(false, Ordering::Relaxed),
                        event::KeyCode::Enter => game
                            .handle_events(SnakeEvent::Movement(Direction::Stop), "just".to_string()),
                        _ => {}
                    },
                    event::Event::Resize(w, h) => buffer.resize(w as usize, h as usize),
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

        game.step();

        //draw
        let r = RectangleShape::new(
            0,
            0,
            (gw + 2) as isize,
            (gh + 2) as isize,
            RenderChar::new('#', Colors{foreground: Some(Color::Blue), background: None}),
            false,
        );
        buffer.draw(&r);

        for food in game.get_food() {
            let r = RectangleShape::new(
                (food.0 + 1) as isize,
                (food.1 + 1) as isize,
                1,
                1,
                RenderChar::new('*', Colors{foreground: Some(Color::Red), background: None}),
                false,
            );
            buffer.draw(&r);
        }

        let my_snake = game.get_snake("just".to_string());
        for part in my_snake.body {
            let r = RectangleShape::new(
                (part.0 + 1) as isize,
                (part.1 + 1) as isize,
                1,
                1,
                RenderChar::new('0', Colors{foreground: Some(Color::Yellow), background: None}),
                false,
            );
            buffer.draw(&r);
        }

        let r = RectangleShape::new(
            (my_snake.head.0 + 1) as isize,
            (my_snake.head.1 + 1) as isize,
            1,
            1,
            RenderChar::new('@', Colors{foreground: Some(Color::Green), background: None}),
            false,
        );
        buffer.draw(&r);

        clear_term();
        buffer.render_to(&mut stdout);

        buffer.clear(RenderChar::empty());

        thread::sleep(Duration::from_millis(1000 / 10));
    }
    stdout.execute(event::DisableMouseCapture).unwrap();
    terminal::disable_raw_mode().unwrap();
    // print!("\x1B[2J");
}
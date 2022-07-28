use console::{Key, Term};
use std::io::Write;
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc;

mod logic;
mod render;

use logic::*;
use render::renderscreen::RenderScreen;
use render::text::Text;

#[tokio::main]
async fn main() {
    // Init stuff
    let mut term = Term::stdout();
    term.hide_cursor().unwrap();
    let (width, height) = (term.size().1 as usize, term.size().0 as usize);
    let mut screen = RenderScreen::new(width, height, "Snake".to_string());
    let (tx, mut rx) = mpsc::channel::<Input>(32);
    let (mut x, mut y) = (0,0);
    let term_input = term.clone();
    // spawn user input system
    tokio::spawn(async move {
        loop {
            let input_key = term_input.read_key().unwrap();
            let input: Input = match input_key {
                Key::Char(input) => match input {
                    'w' => Input::Movement(Direction::Up),
                    'a' => Input::Movement(Direction::Left),
                    's' => Input::Movement(Direction::Down),
                    'd' => Input::Movement(Direction::Rigth),
                    '\u{4}' => Input::Signal(Signal::Disconnect),
                    _ => Input::Nothing,
                },
                Key::ArrowUp => Input::Movement(Direction::Up),
                Key::ArrowLeft => Input::Movement(Direction::Left),
                Key::ArrowDown => Input::Movement(Direction::Down),
                Key::ArrowRight => Input::Movement(Direction::Rigth),
                _ => Input::Nothing,
            };
            tx.send(input).await.unwrap();
        }
    });
    // Game Loop
    'game_loop: loop {
        if let Ok(input) = rx.try_recv() {
            match input {
                Input::Nothing => {
                    // term.write_line(format!("Nothing was input").as_str())
                    //     .unwrap();
                }
                Input::Signal(signal) => match signal {
                    Signal::Disconnect => {
                        break 'game_loop;
                    }
                },
                Input::Movement(direction) => {
                    // let text = Text::new(5, 5, format!("Went: {:?}", direction));
                    // // let text = Text::new(5, 5, format!("Went: {:?}", SystemTime::now()));
                    // screen.draw(&text);
                    match direction {
                        Direction::Up => {y -= 1},
                        Direction::Left => {x -= 1},
                        Direction::Down => {y += 1},
                        Direction::Rigth => {x += 1},
                    }
                }
            }
        }
        // get input
        // do logic

        screen.get_buffer_mut().put(x, y, '#').unwrap();

        term.clear_screen().unwrap();
        term.write_all(screen.as_string().as_bytes()).unwrap();
        screen.clear_buffer(' ');

        thread::sleep(Duration::from_millis(1000 / 60));
    }
    term.show_cursor().unwrap();
}

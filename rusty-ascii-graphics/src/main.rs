use std::io::{stdout, BufWriter, Write};

use crossterm::{
    cursor,
    event::{self, Event},
    style::{Color, Colors},
    terminal::{self, SetTitle},
    ExecutableCommand,
};
use rusty_ascii_graphics::{RenderBuffer, RenderChar};

fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    ((x2 - x1).powf(2f64) + (y2 - y1).powf(2f64)).sqrt()
}

fn main() {
    terminal::enable_raw_mode().unwrap();
    let mut stdout = BufWriter::new(stdout());
    stdout.execute(cursor::Hide).unwrap();
    stdout.execute(event::EnableMouseCapture).unwrap();
    stdout.execute(terminal::EnterAlternateScreen).unwrap();

    let (width, height) = (
        terminal::size().unwrap().0 as usize,
        terminal::size().unwrap().1 as usize,
    );
    let mut buffer = RenderBuffer::new(width, height);

    let fd = distance(width as f64, height as f64, 0f64, 0f64);

    for x in 0..width {
        for y in 0..height {
            buffer.put(
                x as isize,
                y as isize,
                RenderChar::new(
                    ' ',
                    Colors {
                        foreground: None,
                        background: Some(Color::Rgb {
                            r: (255 as f64
                                / (fd / distance(x as f64, y as f64, width as f64, height as f64)))
                                as u8,
                            g: (255 as f64 / (width as f64 / x as f64)) as u8,
                            b: (255 as f64 / (height as f64 / y as f64)) as u8,
                        }),
                    },
                ),
            )
        }
    }

    buffer.render_to(&mut stdout);

    stdout.flush().unwrap();

    loop {
        // `read()` blocks until an `Event` is available
        match event::read().unwrap() {
            Event::Key(_) => {}
            Event::Mouse(event) => match event.kind {
                event::MouseEventKind::Down(_) => {
                    let x = event.column;
                    let y = event.row;
                    stdout.execute(SetTitle(format!("{}:{}", x, y))).unwrap();
                }
                _ => {}
            },
            Event::Resize(_, _) => break,
        }
    }

    stdout.execute(terminal::LeaveAlternateScreen).unwrap();
    stdout.execute(event::DisableMouseCapture).unwrap();
    stdout.execute(cursor::Show).unwrap();
    terminal::disable_raw_mode().unwrap();
}

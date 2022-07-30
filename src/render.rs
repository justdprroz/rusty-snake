use std::io::{Stdout, Write};

use crossterm::{
    style::{Colors, ResetColor, SetColors},
    ExecutableCommand,
};

/// Traits that require object to be rendered on given buffer
pub trait Drawable {
    fn draw(&self, render: &mut RenderBuffer);
}

#[derive(Clone)]
pub struct RenderChar {
    char: char,
    colors: Colors,
}

impl RenderChar {
    pub fn new(char: char, colors: Colors) -> RenderChar {
        RenderChar {
            char: char,
            colors: colors,
        }
    }
    pub fn empty() -> RenderChar {
        RenderChar {
            char: ' ',
            colors: Colors {
                foreground: None,
                background: None,
            },
        }
    }
}

/// Main RenderBuffer struct
pub struct RenderBuffer {
    width: usize,
    height: usize,
    buffer: Vec<RenderChar>,
}

impl RenderBuffer {
    /// Build new RenderBuffer from sizes.
    pub fn new(width: usize, height: usize) -> RenderBuffer {
        RenderBuffer {
            width: width,
            height: height,
            buffer: vec![RenderChar::empty(); width * height],
        }
    }

    /// Build new RenderBuffer from copying another.
    pub fn from(from: &RenderBuffer) -> RenderBuffer {
        RenderBuffer {
            width: from.width,
            height: from.height,
            buffer: from.buffer.clone(),
        }
    }

    /// Put char at specified position.
    pub fn put(&mut self, x: isize, y: isize, v: RenderChar) {
        if x >= 0 && y >= 0 {
            let x = x as usize;
            let y = y as usize;
            if x < self.width && y < self.height {
                self.buffer[x + y * self.width] = v;
            } else {
            }
        }
    }

    /// Get char at specified position. Return ' ' if outside of buffer
    pub fn get(&self, x: isize, y: isize) -> RenderChar {
        if x >= 0 && y >= 0 {
            let x = x as usize;
            let y = y as usize;
            if x < self.width && y < self.height {
                self.buffer[x + y * self.width].clone()
            } else {
                RenderChar::empty()
            }
        } else {
            RenderChar::empty()
        }
    }

    /// Convert RenderBuffer to printable String.
    pub fn to_string(&self) -> String {
        // self.buffer.iter().collect()
        "no".to_string()
    }

    /// Fill every position with specified char
    pub fn clear(&mut self, char_to_fill: RenderChar) {
        self.buffer
            .iter_mut()
            .map(|x| *x = char_to_fill.clone())
            .count();
    }

    /// Draw Drawable object at this buffer
    pub fn draw<T: Drawable>(&mut self, object: &T) {
        object.draw(self);
    }

    /// Change buffer sizes and recreate char array
    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.buffer = vec![RenderChar::empty(); self.width * self.height]
    }

    pub fn render_to<T: std::io::Write>(&self, stdout: &mut T) {
        // stdout
        //     .write_all(&self.buffer.iter().map(|x| *x as u8).collect::<Vec<u8>>())
        //     .unwrap();
        let mut colors = Colors {
            foreground: None,
            background: None,
        };
        let mut buf: Vec<u8> = Vec::with_capacity(1024);
        for char in &self.buffer {
            if char.colors != colors {
                stdout.write_all(&buf).unwrap();
                buf.clear();
                colors = char.colors;
                stdout.execute(SetColors(colors)).unwrap();
            }
            buf.push(char.char as u8);
        }
        stdout.write_all(&buf).unwrap();
        stdout.execute(ResetColor).unwrap();
    }
}

/// just pixel struct
pub struct Point {
    x: isize,
    y: isize,
    c: RenderChar,
}

impl Point {
    pub fn new(x: isize, y: isize, c: RenderChar) -> Point {
        Point { x, y, c }
    }
}

impl Drawable for Point {
    fn draw(&self, render: &mut RenderBuffer) {
        render.put(self.x, self.y, self.c.clone());
    }
}

/// Rectangle shape
pub struct RectangleShape {
    x: isize,
    y: isize,
    width: isize,
    height: isize,
    fill_char: RenderChar,
    do_fill: bool,
}

impl RectangleShape {
    pub fn new(x: isize, y: isize, w: isize, h: isize, c: RenderChar, f: bool) -> RectangleShape {
        RectangleShape {
            x: x,
            y: y,
            width: w,
            height: h,
            fill_char: c,
            do_fill: f,
        }
    }
}

impl Drawable for RectangleShape {
    fn draw(&self, render: &mut RenderBuffer) {
        if self.do_fill {
            for x in 0..self.width {
                for y in 0..self.height {
                    render.put(self.x + x, self.y + y, self.fill_char.clone());
                }
            }
        } else {
            for x in 0..self.width {
                render.put(self.x + x, self.y + 0, self.fill_char.clone());
                render.put(self.x + x, self.y + self.height - 1, self.fill_char.clone());
            }
            for y in 0..self.height {
                render.put(self.x + 0, self.y + y, self.fill_char.clone());
                render.put(self.x + self.width - 1, self.y + y, self.fill_char.clone());
            }
        }
    }
}

/// Text Struct
pub struct Text {
    x: isize,
    y: isize,
    text: String,
}

impl Text {
    pub fn new(x: isize, y: isize, s: String) -> Text {
        Text {
            x: x,
            y: y,
            text: s,
        }
    }
    pub fn set_text(&mut self, s: String) {
        self.text = s;
    }
}

impl Drawable for Text {
    fn draw(&self, render: &mut RenderBuffer) {
        let mut cx = self.x;
        let mut cy = self.y;
        for char in self.text.chars() {
            if char == '\n' {
                cy += 1;
                cx = self.x;
            } else {
                render.put(
                    cx,
                    cy,
                    RenderChar::new(
                        char,
                        Colors {
                            foreground: None,
                            background: None,
                        },
                    ),
                );
                cx += 1;
            }
        }
    }
}

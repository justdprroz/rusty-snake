use super::{traits::Drawable, renderbuffer::RenderBuffer};
pub struct Text {
    x: usize,
    y: usize,
    text: String
}

impl Text {
    pub fn new(x: usize, y: usize, s: String) -> Text {
        Text {
            x: x,
            y: y,
            text: s
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
                render.put(cx, cy, char).unwrap();
                cx += 1;
            }
        }
    }
}
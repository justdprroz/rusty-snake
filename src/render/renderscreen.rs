use super::renderbuffer::RenderBuffer;
use super::traits::Drawable;
pub struct RenderScreen {
    buffer: RenderBuffer,
    title: String
}

impl RenderScreen {
    pub fn new(w: usize, h: usize, t: String) -> RenderScreen {
        RenderScreen {
            buffer: RenderBuffer::new(w, h),
            title: t,
        }
    }
    pub fn draw<T: Drawable>(&mut self, object: &T) {
        object.draw(&mut self.buffer);
    }
    pub fn as_string(&self) -> String {
        self.buffer.get_printable_string()
    }
    pub fn clear_buffer(&mut self, value: char) {
        self.buffer.clear(value);
    }
    pub fn get_buffer(&self) -> &RenderBuffer {
        &self.buffer
    }
    pub fn get_buffer_mut(&mut self) -> &mut RenderBuffer {
        &mut self.buffer
    }
}

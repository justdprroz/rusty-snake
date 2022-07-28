use super::renderbuffer::RenderBuffer;
pub trait Drawable {
    fn draw(&self, render: &mut RenderBuffer);
}
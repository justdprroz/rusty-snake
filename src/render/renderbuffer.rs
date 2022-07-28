pub struct RenderBuffer {
    width: usize,
    height: usize,
    buffer: Vec<char>,
}

impl RenderBuffer {
    pub fn new(width: usize, height: usize) -> RenderBuffer {
        RenderBuffer {
            width: width,
            height: height,
            buffer: vec![' '; width * height],
        }
    }
    pub fn from(from: &RenderBuffer) -> RenderBuffer {
        RenderBuffer {
            width: from.width,
            height: from.height,
            buffer: from.buffer.clone(),
        }
    }
    pub fn get_printable_string(&self) -> String {
        self.buffer.iter().collect()
    }
    pub fn put(&mut self, x: usize, y: usize, v: char) -> Result<(), ()> {
        if x < self.width && y < self.height {
            self.buffer[x + y * self.width] = v;
            Ok(())
        } else {
            Err(())
        }
    }
    pub fn get(&self, x: usize, y: usize) -> Result<char, ()> {
        if x < self.width && y < self.height {
            Ok(self.buffer[x + y * self.width])
        } else {
            Err(())
        }
    }
    pub fn clear(&mut self, value: char) {
        self.buffer.iter_mut().map(|x| *x = value).count();
    }
}

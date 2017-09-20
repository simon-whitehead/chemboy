use gameboy::gfx::Color;

#[derive(Clone)]
pub struct Frame {
    pub pixels: Vec<Color>,
}

impl Frame {
    pub fn new() -> Frame {
        Frame { pixels: vec![Color::new(0xFF, 0xFF, 0xFF, 0xFF); ::gameboy::SCREEN_WIDTH * ::gameboy::SCREEN_HEIGHT] }
    }

    pub fn clear(&mut self) {
        self.pixels = vec![Color::new(0xFF, 0xFF, 0xFF, 0xFF); ::gameboy::SCREEN_WIDTH * ::gameboy::SCREEN_HEIGHT];
    }
}

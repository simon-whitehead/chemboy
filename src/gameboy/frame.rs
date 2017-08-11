
#[derive(Clone, Copy, Debug)]
pub struct Color {
    num: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(num: u8, r: u8, g: u8, b: u8) -> Color {
        Color {
            num: num,
            r: r,
            g: g,
            b: b,
        }
    }

    pub fn from_dmg_byte(b: u8) -> Color {
        match b {
            0x00 => Color::new(b, 0xFF, 0xFF, 0xFF),
            0x01 => Color::new(b, 0xC0, 0xC0, 0xC0),
            0x02 => Color::new(b, 0x60, 0x60, 0x60),
            0x03 => Color::new(b, 0x00, 0x00, 0x00),
            _ => panic!("invalid pallete entry: {}", b),
        }
    }
}

#[derive(Clone)]
pub struct Frame {
    pub pixels: Vec<Color>,
}

impl Frame {
    pub fn new() -> Frame {
        Frame { pixels: vec![Color::new(0, 0xFF, 0xFF, 0xFF); 160 * 144] }
    }

    pub fn clear(&mut self) {
        self.pixels = vec![Color::new(0, 0xFF, 0xFF, 0xFF); 160 * 144];
    }
}

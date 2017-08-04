
#[derive(Clone, Copy)]
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
            0x01 => Color::new(b, 0x66, 0x66, 0x66),
            0x02 => Color::new(b, 0x33, 0x33, 0x33),
            0x03 => Color::new(b, 0x00, 0x00, 0x00),
            _ => panic!("invalid pallete entry: {}", b),
        }
    }
}


pub struct Frame {
    pub pixels: [Color; 160 * 144],
}

impl Frame {
    pub fn new() -> Frame {
        Frame {
            pixels: [Color::new(0, 0xFF, 0xFF, 0xFF); 160 * 144],
        }
    }

    pub fn clear(&mut self) {
        self.pixels = [Color::new(0, 0xFF, 0xFF, 0xFF); 160 * 144];
    }
}

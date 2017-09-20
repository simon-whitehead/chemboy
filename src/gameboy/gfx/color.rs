
#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r: r,
            g: g,
            b: b,
            a: a,
        }
    }

    pub fn is_white(&self) -> bool {
        self.r == 0xFF && self.g == 0xFF && self.b == 0xFF && self.a == 0xFF
    }
}

impl From<u8> for Color {
    fn from(b: u8) -> Color {
        match b {
            0x00 => Color::new(0xFF, 0xFF, 0xFF, 0xFF),
            0x01 => Color::new(0xC0, 0xC0, 0xC0, 0xFF),
            0x02 => Color::new(0x60, 0x60, 0x60, 0xFF),
            0x03 => Color::new(0x00, 0x00, 0x00, 0xFF),
            _ => panic!("invalid pallete entry: {}", b),
        }
    }
}

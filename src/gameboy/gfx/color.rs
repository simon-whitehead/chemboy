use gameboy::ui::theme::Theme;

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

    pub fn from(b: u8, theme: &Theme) -> Color {
        match *theme {
            Theme::Default => {
                match b {
                    0x00 => Color::new(0xFF, 0xFF, 0xFF, 0xFF),
                    0x01 => Color::new(0xC0, 0xC0, 0xC0, 0xFF),
                    0x02 => Color::new(0x60, 0x60, 0x60, 0xFF),
                    0x03 => Color::new(0x00, 0x00, 0x00, 0xFF),
                    _ => panic!("invalid pallete entry: {}", b),
                }
            }
            Theme::ClassicDMG => {
                match b {
                    0x00 => Color::new(0x9B, 0xBC, 0x0F, 0xFF),
                    0x01 => Color::new(0x8B, 0xAC, 0x0F, 0xFF),
                    0x02 => Color::new(0x30, 0x62, 0x30, 0xFF),
                    0x03 => Color::new(0x0F, 0x38, 0x0F, 0xFF),
                    _ => panic!("invalid pallete entry: {}", b),
                }
            }
        }
    }
}
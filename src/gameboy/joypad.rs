
pub enum InputLine {
    None,
    Directional,
    Button,
}

#[derive(Debug)]
pub enum JoypadButton {
    Down,
    Start,
    Up,
    Select,
    Left,
    B,
    Right,
    A,
}

pub struct Joypad {
    line: InputLine,

    down: bool,
    start: bool,

    up: bool,
    select: bool,

    left: bool,
    b: bool,

    right: bool,
    a: bool,
}

impl Joypad {
    pub fn new() -> Joypad {
        Joypad {
            line: InputLine::None,

            down: false,
            start: false,
            up: false,
            select: false,
            left: false,
            b: false,
            right: false,
            a: false,
        }
    }

    pub fn as_u8(&self) -> u8 {
        let r = match self.line {
            InputLine::Directional => {
                0x10 | (if self.down { 0x00 } else { 0x08 }) | (if self.up { 0x00 } else { 0x04 }) |
                (if self.left { 0x00 } else { 0x02 }) |
                (if self.right { 0x00 } else { 0x01 })
            }
            InputLine::Button => {
                0x20 | (if self.start { 0x00 } else { 0x08 }) |
                (if self.select { 0x00 } else { 0x04 }) |
                (if self.b { 0x00 } else { 0x02 }) |
                (if self.a { 0x00 } else { 0x01 })
            },
            InputLine::None => 0x30
        };
        println!("Joypad read: {:b}", r);
        r
    }

    pub fn set_input_line(&mut self, line: InputLine) {
        self.line = line;
    }

    pub fn press(&mut self, b: JoypadButton) {
        match b {
            Down => self.down = true,
            Up => self.up = true,
            Left => self.left = true,
            Right => self.right = true,

            Start => self.start = true,
            Select => self.select = true,
            A => self.a = true,
            B => self.b = true
        }
    }

    pub fn unpress(&mut self, b: JoypadButton) {
        match b {
            Down => self.down = false,
            Up => self.up = false,
            Left => self.left = false,
            Right => self.right = false,

            Start => self.start = false,
            Select => self.select = false,
            A => self.a = false,
            B => self.b = false
        }
    }
}
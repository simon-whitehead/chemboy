
#[derive(Debug)]
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

    pub fn from_u8(b: u8) -> Joypad {
        let right_or_a = b & 0x01 == 0x01;
        let left_or_b = b & 0x02 == 0x02;
        let up_or_select = b & 0x04 == 0x04;
        let down_or_start = b & 0x08 == 0x08;

        let directional = b & 0x10 == 0x10;
        let buttons = b & 0x20 == 0x20;

        Joypad {
            line: InputLine::None,

            down: directional && down_or_start,
            start: buttons && down_or_start,

            up: directional && up_or_select,
            select: buttons && up_or_select,

            left: directional && left_or_b,
            b: buttons && left_or_b,

            right: directional && right_or_a,
            a: buttons && right_or_a
        }
    }

    pub fn as_u8(&self) -> u8 {
        let r = match self.line {
            InputLine::Directional => {
                (if self.down { 0x00 } else { 0x08 }) | (if self.up { 0x00 } else { 0x04 }) |
                (if self.left { 0x00 } else { 0x02 }) |
                (if self.right { 0x00 } else { 0x01 })
            }
            InputLine::Button => {
                (if self.start { 0x00 } else { 0x08 }) |
                (if self.select { 0x00 } else { 0x04 }) |
                (if self.b { 0x00 } else { 0x02 }) |
                (if self.a { 0x00 } else { 0x01 })
            },
            InputLine::None =>  {
                0x30 | (if self.down || self.start { 0x00 } else { 0x08 }) | (if self.up || self.select { 0x00 } else { 0x04 }) |
                (if self.left || self.b { 0x00 } else { 0x02 }) |
                (if self.right || self.a { 0x00 } else { 0x01 })
            }
        };
        print!("Joypad read for {:?}: ", self.line);
        for n in 0..8 {
            let v = (r >> n) & 0x01;
            print!("{}", v);
        }
        println!("");
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
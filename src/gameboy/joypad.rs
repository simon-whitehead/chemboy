use gameboy::irq::{Interrupt, Irq};

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
    pub data: u8,
    pub state: u8,
    pub cycles: usize,
}

impl Joypad {
    pub fn new() -> Joypad {
        Joypad {
            data: 0xFF,
            state: 0xFF,
            cycles: 0x00,
        }
    }

    pub fn step(&mut self, irq: &mut Irq, cycles: usize) -> Result<(), String> {
        self.cycles += cycles;

        if self.cycles >= 0x10000 {
            // 64hz
            self.cycles = 0x00;
            self.update(irq);
        }


        Ok(())
    }

    fn update(&mut self, irq: &mut Irq) {
        let mut d = self.data & 0xF0;

        if d & 0x30 == 0x10 {
            // Directional buttons
            d |= self.state & 0x0F;
        } else if d & 0x30 == 0x20 {
            // Non-directional buttons
            d |= self.state & 0x0F;
        } else if d & 0x30 == 0x30 {
            // Both ...
            d |= 0x0F;
        }

        if (self.data & !d & 0x0F) != 0x00 {
            irq.request(Interrupt::Joypad);
        }

        self.data = d;
    }

    pub fn from_u8(&mut self, b: u8, irq: &mut Irq) {
        self.data = (self.data & 0xCF) | (b & 0x30);
        self.update(irq);
    }

    pub fn press(&mut self, b: JoypadButton) {
        match b {
            JoypadButton::Down | JoypadButton::Start => self.state |= 0x08,
            JoypadButton::Up | JoypadButton::Select => self.state |= 0x04,
            JoypadButton::Left | JoypadButton::B => self.state |= 0x02,
            JoypadButton::Right | JoypadButton::A => self.state |= 0x01,
        }
    }

    pub fn unpress(&mut self, b: JoypadButton) {
        match b {
            JoypadButton::Down | JoypadButton::Start => self.state &= !0x08,
            JoypadButton::Up | JoypadButton::Select => self.state &= !0x04,
            JoypadButton::Left | JoypadButton::B => self.state &= !0x02,
            JoypadButton::Right | JoypadButton::A => self.state &= !0x01,
        }
    }
}

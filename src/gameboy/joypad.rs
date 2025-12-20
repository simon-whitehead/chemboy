use crate::gameboy;
use gameboy::irq::{Interrupt, Irq};

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
        let prev = self.data;

        let select = self.data & 0x30;
        let mut lower_nibble = 0x0F;

        if (select & 0x10) == 0 {
            // D-pad selected
            lower_nibble &= self.state & 0x0F;
        }

        if (select & 0x20) == 0 {
            // Buttons selected
            lower_nibble &= self.state >> 4;
        }

        let d = 0xC0 | select | lower_nibble;

        // Joypad interrupt: trigger on 1->0 transitions in low nibble (new press)
        if ((prev & 0x0F) & !(d & 0x0F)) != 0 {
            irq.request(Interrupt::Joypad);
        }

        self.data = d;
    }

    pub fn from_u8(&mut self, b: u8, irq: &mut Irq) {
        self.data = (self.data & !0x30) | (b & 0x30);
        self.update(irq);
    }

    pub fn unpress(&mut self, b: JoypadButton, irq: &mut Irq) {
        match b {
            JoypadButton::Down => self.state |= 0x08,
            JoypadButton::Start => self.state |= 0x80,
            JoypadButton::Up => self.state |= 0x04,
            JoypadButton::Select => self.state |= 0x40,
            JoypadButton::Left => self.state |= 0x02,
            JoypadButton::B => self.state |= 0x20,
            JoypadButton::Right => self.state |= 0x01,
            JoypadButton::A => self.state |= 0x10,
        }

        self.update(irq);
    }

    pub fn press(&mut self, b: JoypadButton, irq: &mut Irq) {
        match b {
            JoypadButton::Down => self.state &= !0x08,
            JoypadButton::Start => self.state &= !0x80,
            JoypadButton::Up => self.state &= !0x04,
            JoypadButton::Select => self.state &= !0x40,
            JoypadButton::Left => self.state &= !0x02,
            JoypadButton::B => self.state &= !0x20,
            JoypadButton::Right => self.state &= !0x01,
            JoypadButton::A => self.state &= !0x10,
        }

        self.update(irq);
    }
}

use gameboy::{Interconnect, Interrupt, Irq, Memory};

const VRAM_SIZE: usize = 0x4000;
const SPRITE_DATA_SIZE: usize = 0xA0;

struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Color {
        Color { r: r, g: g, b: b }
    }

    pub fn from_byte(b: u8) -> Color {
        match b {
            0x00 => Color::new(0xFF, 0xFF, 0xFF),
            0x01 => Color::new(0x66, 0x66, 0x66),
            0x02 => Color::new(0x33, 0x33, 0x33),
            0x03 => Color::new(0x00, 0x00, 0x00),
            _ => panic!("invalid pallete entry"),
        }
    }
}

pub struct Gpu {
    pub enabled: bool,
    pub ram: Memory,
    pub sprite_data: Memory,
    control_register: u8,
    stat: u8,
    scroll_y: u8,
    scroll_x: u8,
    window_y: u8,
    window_x: u8,
    ly: u8,
    lyc: u8,
    bg_palette: u8,
    palette0: u8,
    palette1: u8,

    cycles: isize,
}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu {
            enabled: true,
            ram: Memory::new(VRAM_SIZE),
            sprite_data: Memory::new(SPRITE_DATA_SIZE),
            control_register: 0x00,
            stat: 0x00,
            scroll_y: 0x00,
            scroll_x: 0x00,
            window_y: 0x00,
            window_x: 0x00,
            ly: 0x00,
            lyc: 0x00,
            bg_palette: 0x00,
            palette0: 0x00,
            palette1: 0x00,
            cycles: 0x00,
        }
    }

    pub fn step(&mut self, irq: &mut Irq, cycles: usize) {
        let cycles = cycles as isize;
        self.enabled = self.control_register & 0x80 == 0x80;

        if !self.enabled {
            return;
        }

        self.cycles -= cycles;
        if self.cycles < 0 {
            self.cycles = 0x1C8; // it takes 456 CPU clock cycles to draw 1 LCD scanline
            self.ly = (self.ly + 0x01) % 0x9A; // LY can only be within 0...153
            if self.ly >= 0x90 {
                // V-Blank
                irq.request(Interrupt::Vblank);
            }
            self.check_coincidence();
            if self.get_coincidence_flag() && self.coincidence_interrupt_enabled() {
                irq.request(Interrupt::Lcd);
            }
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            0x40 => self.control_register,
            0x41 => self.stat,
            0x42 => self.scroll_y,
            0x43 => self.scroll_x,
            0x4A => self.window_y,
            0x4B => self.window_x,
            0x44 => self.ly,
            0x45 => self.lyc,
            0x47 => self.bg_palette,
            0x48 => self.palette0,
            0x49 => self.palette1,
            _ => panic!("tried to read GPU memory that is not mapped"),
        }
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        match addr {
            0x40 => self.control_register = val,
            0x41 => self.stat = val,
            0x42 => self.scroll_y = val,
            0x43 => self.scroll_x = val,
            0x4A => self.window_y = val,
            0x4B => self.window_x = val,
            0x44 => self.ly = val,
            0x45 => {
                self.lyc = val;
                self.check_coincidence();
            }
            0x47 => self.bg_palette = val,
            0x48 => self.palette0 = val,
            0x49 => self.palette1 = val,
            _ => panic!("tried to write GPU memory that is not mapped: {:04}", addr),
        }
    }

    fn check_coincidence(&mut self) {
        // If LY == LYC then set the coincidence flag
        if self.lyc == self.ly {
            self.set_coincidence_flag(true);
        } else {
            self.set_coincidence_flag(false);
        }
    }

    fn set_coincidence_flag(&mut self, set: bool) {
        if set {
            self.stat |= 0x04;
        } else {
            self.stat &= !0x04;
        }
    }

    fn get_coincidence_flag(&self) -> bool {
        self.stat & 0x04 == 0x04
    }

    fn set_coincidence_interrupt(&mut self, set: bool) {
        if set {
            self.stat |= 0x20;
        } else {
            self.stat &= !0x20;
        }
    }

    fn coincidence_interrupt_enabled(&self) -> bool {
        self.stat & 0x20 == 0x20
    }
}

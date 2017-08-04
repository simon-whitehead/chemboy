use gameboy::{Interconnect, Interrupt, Irq, Memory};
use gameboy::frame::{Color, Frame};

const VRAM_SIZE: usize = 0x4000;
const SPRITE_DATA_SIZE: usize = 0xA0;

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

    pub frame: Frame,
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
            frame: Frame::new(),
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

    pub fn render_frame(&mut self) {
        if !self.enabled || self.ly >= 144 {
            return;
        }
        self.render_background();
    }

    fn render_background(&mut self) {
        let background_map_base_address = self.get_base_background_map_address() as usize;
        let tile_base_address = self.get_base_tile_address() as usize;
        let line = self.ly.wrapping_add(self.scroll_y) as usize;
        let bg_map_row = (line / 0x08) as usize;
        for i in 0..160 {
            let x = (i as u8).wrapping_add(self.scroll_x);
            let bg_map_col = (x / 8) as usize;
            let raw_tile_number =
                self.ram[background_map_base_address + (bg_map_row * 0x20 + bg_map_col)];
            let t = if tile_base_address == 0x0000 {
                raw_tile_number as usize
            } else {
                128 + ((raw_tile_number as i8 as i16) + 128) as usize
            };
            let line = (line % 0x08) << 0x01;
            let tile_data_start = tile_base_address; // + (t * 0x10) + line;
            let x_shift = (x % 8).wrapping_sub(7).wrapping_mul(0xFF);
            let tile_data1 = self.ram[tile_data_start] >> x_shift;
            let tile_data2 = self.ram[tile_data_start + 0x01] >> x_shift;
            let total_row_data = (tile_data1 << 1) | tile_data2;
            let color_value = total_row_data & 0x03;
            if color_value > 3 {
                println!("Gonna panic. X: {}, total_row_data: {:b}, shifted: {:b}, color_value: {:b}", x, total_row_data, (total_row_data >> (15 - x)), (total_row_data >> (15 - x)) & 0x03);
            }
            //let color_value = (t1 as u16).wrapping_mul((0x0E as u16).wrapping_sub(x as u16 * 2));
            let c = Color::from_dmg_byte(color_value as u8);
            //println!("Writing pixel to: {}", self.ly as usize * 160 + i as usize);
            self.frame.pixels[self.ly as usize * 160 + i as usize] = c;
        }
    }

    fn get_base_tile_address(&self) -> u16 {
        if self.control_register & 0x10 == 0x10 {
            println!("base tile address 0x8000");
            0x0000
        } else {
            0x0800
        }
    }

    fn get_base_background_map_address(&self) -> u16 {
        if self.control_register & 0x08 == 0x08 {
            0x1C00
        } else {
            0x1800
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

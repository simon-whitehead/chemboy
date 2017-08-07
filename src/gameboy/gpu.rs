use gameboy::{Interconnect, Interrupt, Irq, Memory};
use gameboy::frame::{Color, Frame};

const VRAM_SIZE: usize = 0x4000;
const SPRITE_DATA_SIZE: usize = 0xA0;

pub struct GpuStat {
    pub coincidence_interrupt_enabled: bool,
    pub OAM_interrupt_enabled: bool,
    pub VBlank_interrupt_enabled: bool,
    pub HBlank_interrupt_enabled: bool,
}

impl GpuStat {
    pub fn new() -> GpuStat {
        GpuStat {
            coincidence_interrupt_enabled: false,
            OAM_interrupt_enabled: false,
            VBlank_interrupt_enabled: false,
            HBlank_interrupt_enabled: false,
        }
    }

    pub fn from_u8(b: u8) -> GpuStat {
        GpuStat {
            coincidence_interrupt_enabled: b & 0x40 == 0x40,
            OAM_interrupt_enabled: b & 0x20 == 0x20,
            VBlank_interrupt_enabled: b & 0x10 == 0x10,
            HBlank_interrupt_enabled: b & 0x08 == 0x08,
        }
    }

    pub fn to_u8(&self, gpu: &Gpu) -> u8 {
        (if self.coincidence_interrupt_enabled {
             0x40
         } else {
             0
         }) | (if self.OAM_interrupt_enabled { 0x20 } else { 0 }) |
            (if self.VBlank_interrupt_enabled {
                 0x10
             } else {
                 0
             }) |
            (if self.HBlank_interrupt_enabled {
                 0x08
             } else {
                 0
             }) | (if gpu.ly == gpu.lyc { 0x04 } else { 0 }) | gpu.mode.to_u8()
    }
}

pub enum GpuMode {
    HBlank,
    VBlank,
    SearchingRam,
    TransferringData,
}

impl GpuMode {
    fn cycles(&self, scroll_x: u8) -> isize {
        let scroll_adjust = match scroll_x % 0x08 {
            5...7 => 2,
            1...4 => 1,
            _ => 0,
        };
        match *self {
            GpuMode::SearchingRam => 21,
            GpuMode::TransferringData => 43 + scroll_adjust,
            GpuMode::HBlank => 50 - scroll_adjust,
            GpuMode::VBlank => 114,
        }
    }

    fn to_u8(&self) -> u8 {
        match *self {
            GpuMode::HBlank => 0x00,
            GpuMode::VBlank => 0x01,
            GpuMode::SearchingRam => 0x02,
            GpuMode::TransferringData => 0x03,
        }
    }
}

pub struct Gpu {
    pub enabled: bool,
    pub ram: Memory,
    pub sprite_data: Memory,
    control_register: u8,
    stat: GpuStat,
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
    mode: GpuMode,

    counter: u8,
    tile_base: usize,
    background_base: usize,
}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu {
            enabled: true,
            ram: Memory::new(VRAM_SIZE),
            sprite_data: Memory::new(SPRITE_DATA_SIZE),
            control_register: 0x00,
            stat: GpuStat::new(),
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
            mode: GpuMode::HBlank,
            counter: 0,
            tile_base: 0x00,
            background_base: 0xC00,
        }
    }

    pub fn reset(&mut self) {
        *self = Gpu::new();
    }

    pub fn step(&mut self, irq: &mut Irq, cycles: usize) -> Result<(), String> {
        let cycles = cycles as isize;

        if !self.enabled {
            return Ok(());
        }

        self.cycles += cycles;
        // self.cycles = 0x1C8; // it takes 456 CPU clock cycles to draw 1 LCD scanline
        match self.mode {
            GpuMode::HBlank => {
                if self.cycles >= 0xCC {
                    self.cycles = 0x00;
                    self.ly = self.ly + 0x01;
                    if self.ly >= 0x90 {
                        self.switch_mode(GpuMode::VBlank, irq);
                    } else {
                        self.switch_mode(GpuMode::SearchingRam, irq);
                    }
                    self.check_coincidence(irq);
                }
            }
            GpuMode::TransferringData => {
                if self.cycles >= 0xAC {
                    self.render_scanline();

                    self.cycles = 0x00;
                    self.switch_mode(GpuMode::HBlank, irq);
                }
            }
            GpuMode::SearchingRam => {
                if self.cycles >= 0x50 {
                    self.cycles = 0x00;
                    self.switch_mode(GpuMode::TransferringData, irq);
                }
            }
            GpuMode::VBlank => {
                if self.cycles >= 0x1C8 {
                    self.cycles = 0x00;
                    self.ly += 0x01;
                    if self.ly >= 0x99 {
                        self.ly = 0;
                        self.switch_mode(GpuMode::SearchingRam, irq);
                    }
                    self.check_coincidence(irq);
                }
            }
        }

        Ok(())
    }

    fn render_scanline(&mut self) {
        self.clear_scanline();
        self.render_background();
    }

    fn clear_scanline(&mut self) {
        let line = self.ly.wrapping_add(self.scroll_y) as usize;
        for i in 0..160 {
            self.frame.pixels[line * 160 + i] = Color::new(0, 0xFF, 0xFF, 0xFF);
        }
    }

    fn render_background(&mut self) {
        let background_map_base_address = self.background_base;
        let tile_base_address = self.tile_base;
        let line = self.ly.wrapping_add(self.scroll_y) as usize;
        let bg_map_row = (line / 0x08) as usize;
        for i in 0..160 {
            let x = (i as u8).wrapping_add(self.scroll_x);
            let bg_map_col = (x / 8) as usize;
            let raw_tile_number =
                self.ram[background_map_base_address + (bg_map_row * 0x20 + bg_map_col)];
            let t = if tile_base_address == 0x00 {
                raw_tile_number as usize
            } else {
                128 + ((raw_tile_number as i8 as i16) + 128) as usize
            };

            let line_offset = (line % 0x08) << 0x01;
            let tile_data_start = tile_base_address + (t * 0x10) + line_offset;
            let x_shift = (x % 8).wrapping_sub(0x07).wrapping_mul(0xFF);
            let tile_data1 = (self.ram[tile_data_start] >> x_shift) & 0x01;
            let tile_data2 = (self.ram[tile_data_start + 0x01] >> x_shift) & 0x01;
            let total_row_data = (tile_data2 << 1) | tile_data1;
            let color_value = total_row_data;
            let c = self.get_background_color_for_byte(color_value as u8);
            self.frame.pixels[line as usize * 160 + i as usize] = c;
        }
    }

    fn get_background_color_for_byte(&self, b: u8) -> Color {
        let palette_index = match b {
            0x00 => self.bg_palette & 0x0003,
            0x01 => (self.bg_palette & 0x000C) >> 0x02,
            0x02 => (self.bg_palette & 0x0030) >> 0x04,
            0x03 => (self.bg_palette & 0x00C0) >> 0x06,
            _ => panic!("err: invalid pixel color value found"),
        };

        Color::from_dmg_byte(palette_index as u8)
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            0x40 => self.control_register,
            0x41 => self.stat.to_u8(self),
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
            0x40 => {
                self.control_register = val;
                self.enabled = self.control_register & 0x80 == 0x80;
                self.tile_base = if self.control_register & 0x10 == 0x10 {
                    0x00
                } else {
                    0x800
                };
                self.background_base = if self.control_register & 0x08 == 0x08 {
                    0x1C00
                } else {
                    0x1800
                };
            }
            0x41 => self.stat = GpuStat::from_u8(val),
            0x42 => self.scroll_y = val,
            0x43 => self.scroll_x = val,
            0x44 => {
                self.ly = val;
                println!("WRITING TO LY");
            }
            0x45 => {
                self.lyc = val;
            }
            0x47 => self.bg_palette = val,
            0x48 => self.palette0 = val,
            0x49 => self.palette1 = val,
            0x4A => self.window_y = val,
            0x4B => self.window_x = val,
            _ => panic!("tried to write GPU memory that is not mapped: {:04}", addr),
        }
    }

    fn check_coincidence(&mut self, irq: &mut Irq) {
        // If LY == LYC then set the coincidence flag
        if self.lyc == self.ly && self.stat.coincidence_interrupt_enabled {
            irq.request(Interrupt::Lcd);
        }
    }

    fn switch_mode(&mut self, mode: GpuMode, irq: &mut Irq) {
        self.cycles += mode.cycles(self.scroll_x);
        match mode {
            GpuMode::VBlank => {
                if self.stat.VBlank_interrupt_enabled {
                    irq.request(Interrupt::Vblank);
                }
            }
            GpuMode::SearchingRam => {
                if self.stat.OAM_interrupt_enabled {
                    irq.request(Interrupt::Lcd);
                }
            }
            _ => (),
        }
        self.mode = mode;
    }
}

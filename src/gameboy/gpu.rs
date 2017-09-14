
use byteorder::{ByteOrder, LittleEndian};

use std;

use gameboy;
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

#[derive(Eq, PartialEq, Clone)]
pub enum GpuMode {
    HBlank,
    VBlank,
    SearchingRam,
    TransferringData,
}

impl GpuMode {
    fn to_u8(&self) -> u8 {
        match *self {
            GpuMode::HBlank => 0x00,
            GpuMode::VBlank => 0x01,
            GpuMode::SearchingRam => 0x02,
            GpuMode::TransferringData => 0x03,
        }
    }
}

pub enum SpriteShape {
    Square,
    Rectangle,
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
    pub ly: u8,
    lyc: u8,
    bg_palette: u8,
    palette0: u8,
    palette1: u8,

    cycles: isize,

    pub frame: Frame,
    pub backbuffer: Frame,
    mode: GpuMode,

    counter: u8,
    tile_base: usize,
    background_base: usize,
    sprite_shape: SpriteShape,
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
            backbuffer: Frame::new(),
            mode: GpuMode::HBlank,
            counter: 0,
            tile_base: 0x00,
            background_base: 0xC00,
            sprite_shape: SpriteShape::Square,
        }
    }

    pub fn reset(&mut self) {
        *self = Gpu::new();
    }

    fn set_status(&mut self, irq: &mut Irq) {
        if !self.enabled {
            self.cycles = 0x1C8;
            self.ly = 0x00;
            self.stat = GpuStat::from_u8(0xFD);
        }

        let current_mode = self.mode.clone();

        if self.ly >= 0x90 {
            self.mode = GpuMode::VBlank;
        } else {
            if self.cycles >= 0x178 {
                self.mode = GpuMode::SearchingRam;
            } else if self.cycles >= 0xCC {
                self.mode = GpuMode::TransferringData;
            } else {
                self.mode = GpuMode::HBlank;
            }
        }

        if self.mode != current_mode {
            irq.request(Interrupt::Lcd);
        }

        self.check_coincidence(irq);
    }

    pub fn step(&mut self, irq: &mut Irq, cycles: usize) -> Result<(), String> {
        self.set_status(irq);

        let cycles = cycles as isize;
        if self.enabled {
            self.cycles -= cycles;
        } else {
            return Ok(());
        }

        if self.cycles <= 0 {
            self.ly += 0x01;

            self.cycles = 0x1C8;

            if self.ly == 0x90 {
                irq.request(Interrupt::Vblank);
            } else if self.ly > 0x99 {
                self.ly = 0x00;
            } else if self.ly < 0x90 {
                self.render_scanline();
            }
        }

        Ok(())
    }

    fn render_scanline(&mut self) {
        let line = self.ly.wrapping_add(self.scroll_y) as usize;
        // self.clear_scanline(line);
        self.render_background(line);
        self.render_sprites(line);
        self.frame = self.backbuffer.clone();
    }

    fn clear_scanline(&mut self, line: usize) {
        for i in 0..160 {
            self.backbuffer.pixels[line * 160 + i] = Color::new(0xFF, 0xFF, 0xFF, 0xFF);
        }
    }

    fn render_background(&mut self, line: usize) {
        let bg_base = self.background_base;
        let tile_base = self.tile_base;
        let bg_map_row = (line / 0x08) as usize;
        for i in 0..gameboy::SCREEN_WIDTH {
            let x = (i as u8).wrapping_add(self.scroll_x);
            let bg_map_col = (x / 8) as usize;
            let raw_tile_number = self.ram[bg_base + (bg_map_row * 0x20 + bg_map_col)] as usize;

            let line_offset = (line % 0x08) << 0x01;

            let tile_data_start = tile_base +
                                  (if tile_base == 0x00 {
                raw_tile_number
            } else {
                (raw_tile_number as i8 as i16 + 0x80) as usize
            }) * 0x10 + line_offset;

            let x_shift = (x % 8).wrapping_sub(0x07).wrapping_mul(0xFF);
            let tile_data1 = (self.ram[tile_data_start] >> x_shift) & 0x01;
            let tile_data2 = (self.ram[tile_data_start + 0x01] >> x_shift) & 0x01;
            let total_row_data = (tile_data2 << 1) | tile_data1;
            let color_value = total_row_data;
            let c = self.get_background_color_for_byte(color_value as u8);
            self.backbuffer.pixels[self.ly as usize * 160 + i as usize] = c;
        }
    }

    fn render_sprites(&mut self, line: usize) {
        for i in 0..40 {
            let sprite_table_entry_base = i * 0x04;
            let s_y = self.sprite_data[sprite_table_entry_base] as i16 - 0x10;
            let s_x = self.sprite_data[sprite_table_entry_base + 0x01] as i16 - 0x08;
            let sprite_height = if let SpriteShape::Rectangle = self.sprite_shape {
                0x10
            } else {
                0x08
            };

            // Ignore this scanline if the sprite doesn't sit within it
            // Should we be casting to usize here? If the number is negative the sign bit
            // will be throwing this calculation off entirely - perhaps this is why Kirby
            // disappears when sprite Y position is negative?
            if s_y as usize <= line && (s_y as usize + sprite_height) > line {
                let tile_number = self.sprite_data[sprite_table_entry_base + 0x02] as i16;
                let attributes = self.sprite_data[sprite_table_entry_base + 0x03];
                let above_background = attributes & 0x80 == 0x00;
                let flip_y = attributes & 0x40 == 0x40;
                let flip_x = attributes & 0x20 == 0x20;

                let sprite_y = (line as i16 - s_y) << 0x01;
                let tile_data_start = (tile_number * 0x10) + sprite_y;
                for x in 0..8 {
                    if s_x + x < 0 || s_x + x >= 160 {
                        continue;
                    }
                    let shift = if flip_x { x } else { 0x07 - x };
                    let tile_data1 = (self.ram[tile_data_start as usize] >> shift) & 0x01;
                    let tile_data2 = (self.ram[tile_data_start as usize + 0x01] >> shift) & 0x01;
                    let total_row_data = (tile_data2 << 1) | tile_data1;
                    let color_value = total_row_data;
                    if color_value == 0x00 {
                        continue;
                    }
                    let c = self.get_sprite_color_for_byte(color_value as u8,
                                                           ((attributes & 0x10) >> 0x04) as u8);
                    self.backbuffer.pixels[line as usize * 160 + (s_x as usize + x as usize)] = c;
                }
            }
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

    fn get_sprite_color_for_byte(&self, b: u8, palette_entry: u8) -> Color {
        let palette = if palette_entry == 0x00 {
            self.palette0
        } else {
            self.palette1
        };

        let palette_index = match b {
            0x00 => palette & 0x0003,
            0x01 => (palette & 0x000C) >> 0x02,
            0x02 => (palette & 0x0030) >> 0x04,
            0x03 => (palette & 0x00C0) >> 0x06,
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
                self.sprite_shape = if self.control_register & 0x04 == 0x04 {
                    SpriteShape::Rectangle
                } else {
                    SpriteShape::Square
                };
            }
            0x41 => self.stat = GpuStat::from_u8(val),
            0x42 => self.scroll_y = val,
            0x43 => self.scroll_x = val,
            0x44 => (),
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

    pub fn write_u16(&mut self, addr: u16, val: u16) {
        let addr = addr as usize;
        LittleEndian::write_u16(&mut self.ram[addr..], val)
    }

    fn check_coincidence(&mut self, irq: &mut Irq) {
        // If LY == LYC then set the coincidence flag
        if self.lyc == self.ly && self.stat.coincidence_interrupt_enabled {
            irq.request(Interrupt::Lcd);
        }
    }
}

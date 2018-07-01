use byteorder::{ByteOrder, LittleEndian};

use std;

use gameboy;
use gameboy::{Interconnect, Interrupt, Irq, Memory};
use gameboy::gfx::{Color, Frame, GpuMode, GpuStat, SpriteShape, TileRenderOptions, TileRenderType};
use gameboy::ui::theme::Theme;

pub struct Gpu {
    pub enabled: bool,
    pub ram: Memory,
    pub sprite_data: Memory,

    pub ly: u8,
    pub lyc: u8,

    pub frame: Frame,
    pub backbuffer: Frame,
    pub mode: GpuMode,

    control_register: u8,
    stat: GpuStat,
    scroll_y: u8,
    scroll_x: u8,
    window_y: u8,
    window_x: u8,
    bg_palette: u8,
    palette0: u8,
    palette1: u8,

    cycles: usize,

    counter: u8,
    tile_data_addr: usize,
    background_tilemap_addr: usize,
    window_tilemap_addr: usize,
    sprite_shape: SpriteShape,
    sprites_enabled: bool,
    background_enabled: bool,
    window_enabled: bool,

    pub theme: Theme,
}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu {
            enabled: true,
            ram: Memory::new(gameboy::gfx::VRAM_SIZE),
            sprite_data: Memory::new(gameboy::gfx::SPRITE_DATA_SIZE),
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
            tile_data_addr: 0x00,
            background_tilemap_addr: 0xC00,
            window_tilemap_addr: 0x00,
            sprite_shape: SpriteShape::Square,
            sprites_enabled: true,
            background_enabled: true,
            window_enabled: true,

            theme: Theme::Default,
        }
    }

    pub fn reset(&mut self) {
        *self = Gpu::new();
    }

    pub fn step(&mut self, irq: &mut Irq, cycles: usize) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }

        self.cycles += cycles;

        match self.mode {
            GpuMode::OAM => {
                    if self.cycles >= 0x50 {
                        self.cycles = 0x00;
                        self.mode = GpuMode::VRAM;
                    }
            },
            GpuMode::VRAM => {
                if self.cycles >= 0xAC {
                    self.cycles = 0x00;
                    self.mode = GpuMode::HBlank;
                    if self.enabled {
                        self.render_scanline();
                    }
                }
            },
            GpuMode::HBlank => {
                if self.cycles >= 0xCC {
                    self.cycles = 0x00;
                    self.ly += 0x01;

                    if self.ly == 0x8F {
                        self.mode = GpuMode::VBlank;
                        self.frame = self.backbuffer.clone();
                        irq.request(Interrupt::Vblank);
                    } else {
                        self.mode = GpuMode::OAM;
                    }
                }
            },
            GpuMode::VBlank => {
                if self.cycles >= 0x1C8 {
                    self.cycles = 0x00;
                    self.ly += 0x01;

                    if self.ly > 0x99 {
                        self.mode = GpuMode::OAM;
                        self.ly = 0x00;
                    }
                }
            }
        }

        self.check_coincidence(irq);

        Ok(())
    }

    fn render_scanline(&mut self) {
        self.render_background();
        self.render_window();
        self.render_sprites();
    }

    fn render_background(&mut self) {
        requires!(self.background_enabled);

        let options = TileRenderOptions::new(
            TileRenderType::Background,
            self.ly,
            self.background_tilemap_addr,
            self.tile_data_addr,
        );
        self.render_tile(&options);
    }

    fn render_window(&mut self) {
        requires!(self.window_enabled);
        requires!(self.ly as usize >= self.window_y as usize);

        let options = TileRenderOptions::new(
            TileRenderType::Window,
            self.ly,
            self.window_tilemap_addr,
            self.tile_data_addr,
        );
        self.render_tile(&options);
    }

    fn render_tile(&mut self, options: &TileRenderOptions) {
        let window = variant_equals!(TileRenderType::Window, options.render_type);
        let y = if window {
            options.line as usize
        } else {
            options.line.wrapping_add(self.scroll_y) as usize
        };

        // If rendering a window tile, we need to make sure we offset tile line properly
        let window_offset = if window { self.window_y as usize } else { 0x00 };
        let map_row = ((y - window_offset) / 0x08) as usize;
        let start = if window {
            // Gameboy manual: "With WX = 7, the window is displayed from the left edge of the LCD screen."
            // If we don't do this, the window begins rendering slightly to the right
            self.window_x.wrapping_sub(0x07) as usize
        } else {
            0
        };

        for i in start..gameboy::SCREEN_WIDTH {
            // If we're at the window, lets negate the window X position from where we
            // need to be in the tile map
            let x = if window {
                i as u8 - self.window_x
            } else {
                // Otherwise, scroll the background
                (i as u8).wrapping_add(self.scroll_x)
            };
            let map_col = (x / 0x08) as usize;
            let raw_tile_number = self.ram[options.map_addr + (map_row * 0x20 + map_col)] as usize;

            let line_offset = (y % 0x08) << 0x01;

            let tile_data_start = options.tile_base_addr + (if options.tile_base_addr == 0x00 {
                raw_tile_number
            } else {
                (raw_tile_number as i8 as i16 + 0x80) as usize
            }) * 0x10 + line_offset;

            let x_shift = (x % 8).wrapping_sub(0x07).wrapping_mul(0xFF);
            let color_value = Self::build_palette_index(&self.ram[tile_data_start..], x_shift);
            let c = self.get_background_color_for_byte(color_value as u8);
            self.backbuffer.pixels[self.ly as usize * gameboy::SCREEN_WIDTH + i as usize] = c;
        }
    }

    fn render_sprites(&mut self) {
        requires!(self.sprites_enabled);

        for i in 0..40 {
            let sprite_table_entry_base = i * 0x04;
            let s_y = self.sprite_data[sprite_table_entry_base] as i16 - 0x10;
            let s_x = self.sprite_data[sprite_table_entry_base + 0x01] as i16 - 0x08;
            let sprite_height = if let SpriteShape::Rectangle = self.sprite_shape {
                0x10
            } else {
                0x08
            };

            if s_y <= self.ly as i16 && (s_y + sprite_height as i16) > self.ly as i16 {
                let tile_number = self.sprite_data[sprite_table_entry_base + 0x02] as i16;
                let attributes = self.sprite_data[sprite_table_entry_base + 0x03];
                let above_background = attributes & 0x80 == 0x00;
                let flip_y = attributes & 0x40 == 0x40;
                let flip_x = attributes & 0x20 == 0x20;

                let sprite_line = (self.ly as i16 - s_y) << 0x01;
                let tile_data_start = (tile_number * 0x10) + sprite_line;
                for x in 0..8 {
                    if s_x + x < 0 || s_x + x >= gameboy::SCREEN_WIDTH as i16 {
                        continue;
                    }
                    let shift = if flip_x { x } else { 0x07 - x };
                    let color_value = Self::build_palette_index(
                        &self.ram[tile_data_start as usize..],
                        shift as u8,
                    );
                    if color_value == 0x00 {
                        continue;
                    }
                    let c = self.get_sprite_color_for_byte(
                        color_value as u8,
                        ((attributes & 0x10) >> 0x04) as u8,
                    );
                    let idx =
                        self.ly as usize * gameboy::SCREEN_WIDTH + (s_x as usize + x as usize);
                    if idx > self.backbuffer.pixels.len() - 1 {
                        panic!(format!(
                            "Just tried to place a pixel outside framebuffer size. \
                             Index was {} and backbuffer length is {}. line: {:b}, \
                             gameboy::SCREEN_WIDTH: {:b}, s_x: {:b}, x: {:b}",
                            idx,
                            self.backbuffer.pixels.len(),
                            self.ly,
                            gameboy::SCREEN_WIDTH,
                            s_x,
                            x
                        ));
                    } else {
                        self.backbuffer.pixels[idx] = c;
                    }
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

        Color::from(palette_index as u8, &self.theme)
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

        Color::from(palette_index as u8, &self.theme)
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
                let was_enabled = self.enabled;
                self.control_register = val;
                self.enabled = self.control_register & 0x80 == 0x80;
                self.tile_data_addr = if self.control_register & 0x10 == 0x10 {
                    0x00
                } else {
                    0x800
                };
                self.window_tilemap_addr = if self.control_register & 0x40 == 0x40 {
                    0x1C00
                } else {
                    0x1800
                };
                self.background_tilemap_addr = if self.control_register & 0x08 == 0x08 {
                    0x1C00
                } else {
                    0x1800
                };
                self.sprite_shape = if self.control_register & 0x04 == 0x04 {
                    SpriteShape::Rectangle
                } else {
                    SpriteShape::Square
                };
                self.window_enabled = self.control_register & 0x20 == 0x20;
                self.sprites_enabled = self.control_register & 0x02 == 0x02;
                self.background_enabled = self.control_register & 0x01 == 0x01;

                if was_enabled && !self.enabled {
                    self.cycles = 0x00;
                    self.ly = 0x00;
                    self.mode = GpuMode::HBlank;
                    self.frame = Frame::new();
                }
            }
            0x41 => self.stat = GpuStat::from(val),
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

    fn build_palette_index(data: &[u8], x_shift: u8) -> u8 {
        let tile_data1 = (data[0x00] >> x_shift) & 0x01;
        let tile_data2 = (data[0x01] >> x_shift) & 0x01;
        let pixel_palette_entry = (tile_data2 << 1) | tile_data1;
        pixel_palette_entry
    }
}

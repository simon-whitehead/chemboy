
use std::ops::Range;

use gameboy::Memory;
use gameboy::mbc::mbc::get_ram_size;
use gameboy::mbc::MBC;

pub struct MBC1 {
    pub rom: Memory,
    pub ram: Memory,
    ram_enabled: bool,
}

impl MBC1 {
    pub fn new(rom: &[u8]) -> MBC1 {
        let mut r = Memory::new(rom.len());
        r.write_bytes(0x00, rom);

        let ram_size = get_ram_size(rom[0x149]);
        let ram = Memory::new(ram_size);

        MBC1 {
            rom: r,
            ram: ram,
            ram_enabled: true,
        }
    }
}

impl MBC for MBC1 {
    fn read_ram_u8(&self, addr: u16) -> u8 {
        if !self.ram_enabled {
            return 0xFF;
        }
        self.ram[addr as usize]
    }

    fn read_rom_u8(&self, addr: u16) -> u8 {
        self.rom[addr as usize]
    }

    fn read_ram_u16(&self, addr: u16) -> u16 {
        if !self.ram_enabled {
            return 0xFFFF;
        }
        self.ram.read_u16(addr)
    }

    fn read_rom_u16(&self, addr: u16) -> u16 {
        self.rom.read_u16(addr)
    }

    fn write_ram_u8(&mut self, addr: u16, b: u8) {
        if !self.ram_enabled {
            ()
        }
    }

    fn write_rom_u8(&mut self, addr: u16, b: u8) {
        match addr {
            0x0000...0x1FFF => self.ram_enabled = b & 0x0F == 0x0A,
            _ => {
                panic!("Unsupported address range in Memory Bank Controller 1: {:04X}",
                       addr)
            }
        }
    }
}

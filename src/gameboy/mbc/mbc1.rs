
use std::ops::Range;

use gameboy::Memory;
use gameboy::mbc::mbc::get_ram_size;
use gameboy::mbc::MBC;

pub enum BankMode {
    RomBanking,
    RamBanking,
}

pub struct MBC1 {
    pub rom: Memory,
    pub ram: Memory,

    ram_enabled: bool,
    rom_bank: usize,
    ram_bank: usize,
    bank_mode: BankMode,
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
            rom_bank: 0x00,
            ram_bank: 0x00,
            bank_mode: BankMode::RomBanking,
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
        let addr = if addr < 0x4000 {
            addr as usize
        } else {
            self.rom_bank * 0x4000 | (addr as usize & 0x3FFF)
        };

        self.rom[addr]
    }

    fn read_ram_u16(&self, addr: u16) -> u16 {
        if !self.ram_enabled {
            return 0xFFFF;
        }
        self.ram.read_u16(addr)
    }

    fn read_rom_u16(&self, addr: u16) -> u16 {
        println!("Reading u16 from ROM Bank: {}", self.rom_bank);
        let addr = if addr < 0x4000 {
            addr
        } else {
            self.rom_bank as u16 * 0x4000 | (addr & 0x3FFF)
        };

        let addr = addr as usize;
        let a = self.rom[addr] as u16;
        let b = self.rom[addr + 0x01] as u16;
        let result = (b << 0x08) | a;
        println!("Read result: {:04X}", result);
        result
    }

    fn write_ram_u8(&mut self, addr: u16, b: u8) {
        if !self.ram_enabled {
            ()
        }
    }

    fn write_rom_u8(&mut self, addr: u16, b: u8) {
        match addr {
            0x0000...0x1FFF => self.ram_enabled = b == 0x0A,
            0x2000...0x3FFF => {
                let b = if b & 0x1F == 0x00 { 0x01 } else { b & 0x1F };
                self.rom_bank = (self.rom_bank & 0x60) | b as usize;
            }
            0x4000...0x5FFF => {
                if let BankMode::RomBanking = self.bank_mode {
                    self.rom_bank = self.rom_bank & 0x1F | (((b as usize) & 0x03) << 0x05);
                }
            }
            0x6000...0x7FFF => {
                self.bank_mode = if b & 0x01 == 0x01 {
                    BankMode::RamBanking
                } else {
                    BankMode::RomBanking
                };
            }
            _ => {
                panic!("Unsupported address range in Memory Bank Controller 1: {:04X}",
                       addr)
            }
        }
    }

    fn write_ram_u16(&mut self, addr: u16, b: u16) {
        if !self.ram_enabled {
            ()
        }
        self.ram.write_u16(addr, b)
    }
}

use crate::gameboy;
use gameboy::Memory;
use gameboy::mbc::mbc::get_ram_size;
use gameboy::mbc::MBC;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BankMode {
    RomBanking, // mode 0
    RamBanking, // mode 1
}

pub struct MBC1 {
    pub rom: Memory,
    pub ram: Memory,

    ram_enabled: bool,

    // MBC1 registers
    bank1: u8, // 5 bits, values 1..31 (0 becomes 1)
    bank2: u8, // 2 bits, values 0..3
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
            ram,
            ram_enabled: false,
            bank1: 0x01,
            bank2: 0x00,
            bank_mode: BankMode::RomBanking,
        }
    }

    #[inline]
    fn rom_bank_count(&self) -> usize {
        // 16KB banks
        let banks = self.rom.len() / 0x4000;
        if banks == 0 { 1 } else { banks }
    }

    #[inline]
    fn ram_bank_count(&self) -> usize {
        // 8KB banks
        let banks = self.ram.len() / 0x2000;
        if banks == 0 { 0 } else { banks }
    }

    #[inline]
    fn effective_rom_bank_0000(&self) -> usize {
        // Mode 0: fixed bank 0
        // Mode 1: bank depends on bank2 (0x00/0x20/0x40/0x60)
        match self.bank_mode {
            BankMode::RomBanking => 0,
            BankMode::RamBanking => ((self.bank2 as usize) << 5) % self.rom_bank_count(),
        }
    }

    #[inline]
    fn effective_rom_bank_4000(&self) -> usize {
        // 0x4000-0x7FFF always uses bank1 + bank2
        let mut bank = (((self.bank2 as usize) << 5) | (self.bank1 as usize)) % self.rom_bank_count();

        // Ensure lower 5 bits are not zero (0x00,0x20,0x40,0x60 -> +1)
        if (bank & 0x1F) == 0 {
            bank = (bank + 1) % self.rom_bank_count();
        }
        bank
    }

    #[inline]
    fn effective_ram_bank(&self) -> usize {
        // Mode 0: always bank 0
        // Mode 1: bank2 selects RAM bank (if present)
        if self.ram_bank_count() == 0 {
            return 0;
        }

        match self.bank_mode {
            BankMode::RomBanking => 0,
            BankMode::RamBanking => (self.bank2 as usize) % self.ram_bank_count(),
        }
    }

    #[inline]
    fn ram_index(&self, addr: u16) -> usize {
        let bank = self.effective_ram_bank();
        (bank * 0x2000) | ((addr as usize) & 0x1FFF)
    }

    #[inline]
    fn rom_index_0000(&self, addr: u16) -> usize {
        let bank = self.effective_rom_bank_0000();
        (bank * 0x4000) | ((addr as usize) & 0x3FFF)
    }

    #[inline]
    fn rom_index_4000(&self, addr: u16) -> usize {
        let bank = self.effective_rom_bank_4000();
        (bank * 0x4000) | ((addr as usize) & 0x3FFF)
    }
}

impl MBC for MBC1 {
    fn read_ram_u8(&self, addr: u16) -> u8 {
        if !self.ram_enabled || self.ram.len() == 0 {
            return 0xFF;
        }
        self.ram[self.ram_index(addr)]
    }

    fn read_rom_u8(&self, addr: u16) -> u8 {
        let idx = if addr < 0x4000 {
            self.rom_index_0000(addr)
        } else {
            self.rom_index_4000(addr)
        };
        self.rom[idx]
    }

    fn read_ram_u16(&self, addr: u16) -> u16 {
        // Do as two u8 reads so crossing 0x1FFF->0x2000 behaves correctly
        let lo = self.read_ram_u8(addr) as u16;
        let hi = self.read_ram_u8(addr.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    fn read_rom_u16(&self, addr: u16) -> u16 {
        // Same: boundary-correct
        let lo = self.read_rom_u8(addr) as u16;
        let hi = self.read_rom_u8(addr.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    fn write_ram_u8(&mut self, addr: u16, b: u8) {
        if !self.ram_enabled || self.ram.len() == 0 {
            return;
        }
        let idx = self.ram_index(addr);
        self.ram[idx] = b;
    }

    fn write_ram_u16(&mut self, addr: u16, b: u16) {
        // Two u8 writes to preserve bank boundary behavior
        self.write_ram_u8(addr, (b & 0x00FF) as u8);
        self.write_ram_u8(addr.wrapping_add(1), (b >> 8) as u8);
    }

    fn write_rom_u8(&mut self, addr: u16, b: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enabled = (b & 0x0F) == 0x0A;
            }
            0x2000..=0x3FFF => {
                let v = b & 0x1F;
                self.bank1 = if v == 0 { 1 } else { v };
            }
            0x4000..=0x5FFF => {
                self.bank2 = b & 0x03;
            }
            0x6000..=0x7FFF => {
                self.bank_mode = if (b & 0x01) == 0x01 {
                    BankMode::RamBanking
                } else {
                    BankMode::RomBanking
                };
            }
            _ => {
                panic!(
                    "Unsupported address range in Memory Bank Controller 1: {:04X}",
                    addr
                );
            }
        }
    }
}

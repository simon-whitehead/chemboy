use std::ops::{Deref, DerefMut};

use gameboy::interconnect::Interconnect;
use gameboy::mbc::{MBC, MBC0, MBC1};
use gameboy::Memory;

const CART_MEM_SIZE: usize = 0x200000;
const CART_RAM_SIZE: usize = 0x8000;

pub struct CartridgeDetails {
    pub game_title: String,
    pub cartridge_type: u8,
}

pub struct Cartridge {
    rom_code_size: usize,
    pub mbc: Box<MBC>,
    pub details: CartridgeDetails,
}

impl Cartridge {
    pub fn with_rom(rom: &[u8]) -> Cartridge {
        let details = Self::get_details(&rom);
        let cartridge_type = rom[0x147];
        let mbc = Self::get_controller(cartridge_type, &rom);

        Cartridge {
            rom_code_size: rom.len(),
            mbc: mbc,
            details: details,
        }
    }

    pub fn get_details(rom: &[u8]) -> CartridgeDetails {
        let game_title = String::from_utf8_lossy(&rom[0x134..0x142]).into();
        let cartridge_type = rom[0x147];

        CartridgeDetails {
            game_title: game_title,
            cartridge_type: cartridge_type,
        }
    }

    fn get_controller(b: u8, rom: &[u8]) -> Box<MBC> {
        match b {
            0x00 => Box::new(MBC0::new(rom)),
            0x01...0x03 => Box::new(MBC1::new(rom)),
            _ => panic!("MBC not supported"),
        }
    }
}

impl Deref for Cartridge {
    type Target = Box<MBC>;

    fn deref(&self) -> &Self::Target {
        &self.mbc
    }
}

impl DerefMut for Cartridge {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mbc
    }
}
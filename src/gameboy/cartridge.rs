use std::ops::Deref;

use gameboy::interconnect::Interconnect;

pub struct CartridgeDetails {
    pub game_title: String,
}

pub struct Cartridge {
    pub rom: Box<[u8]>,
}

impl Cartridge {
    pub fn new() -> Cartridge {
        Cartridge {
            rom: vec![].into_boxed_slice(),
        }
    }

    pub fn with_rom(rom: Vec<u8>) -> Cartridge {
        Cartridge {
            rom: rom.into_boxed_slice(),
        }
    }

    pub fn details(&self, interconnect: &Interconnect) -> CartridgeDetails {
        CartridgeDetails {
            game_title: String::from_utf8_lossy(interconnect.read_bytes(0x134..0x142)).into(),
        }
    }
}

impl Deref for Cartridge {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.rom
    }
}

use gameboy::interconnect::Interconnect;
use gameboy::Memory;

const CART_MEM_SIZE: usize = 0x200000;
const CART_RAM_SIZE: usize = 0x8000;

pub struct CartridgeDetails {
    pub game_title: String,
}

pub struct Cartridge {
    pub rom: Memory,
    pub ram: Memory,
}

impl Cartridge {
    pub fn new() -> Cartridge {
        Cartridge {
            rom: Memory::new(CART_MEM_SIZE),
            ram: Memory::new(CART_RAM_SIZE),
        }
    }

    pub fn with_rom(rom: Vec<u8>) -> Cartridge {
        let mut r = Memory::new(CART_MEM_SIZE);
        r.write_bytes(0x00, &rom);

        Cartridge {
            rom: r,
            ram: Memory::new(CART_RAM_SIZE),
        }
    }

    pub fn details(&self, interconnect: &Interconnect) -> CartridgeDetails {
        CartridgeDetails {
            game_title: String::from_utf8_lossy(interconnect.read_bytes(0x134..0x142)).into(),
        }
    }
}

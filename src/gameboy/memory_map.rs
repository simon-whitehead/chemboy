
const CART_ROM_BANK_ZERO_START: u16 = 0x000;
const CART_ROM_BANK_ZERO_END: u16 = 0x3FFF;

const BIOS_START: u16 = 0x0000;
const BIOS_END: u16 = 0x00FF;

const CART_HEADER_START: u16 = 0x0100;
const CART_HEADER_END: u16 = 0x014F;

const CART_ROM_OTHER_BANK_START: u16 = 0x4000;
const CART_ROM_OTHER_BANK_END: u16 = 0x7FFF;

const GFX_RAM_START: u16 = 0x8000;
const GFX_RAM_END: u16 = 0x9FFF;

const CART_RAM_START: u16 = 0xA000;
const CART_RAM_END: u16 = 0xBFFF;

const RAM_START: u16 = 0xC000;
const RAM_END: u16 = 0xDFFF;

const RAM_SHADOW_START: u16 = 0xE000;
const RAM_SHADOW_END: u16 = 0xFDFF;

const GFX_SPRITE_INFORMATION: u16 = 0xFE00;
const GFX_SPRITE_INFO_END: u16 = 0xFE9F;

const IO_START: u16 = 0xFF00;
const IO_END: u16 = 0xFF7F;

const STACK_START: u16 = 0xFF80;
const STACK_END: u16 = 0xFFFF;

pub enum Address {
    Bios(u8),
    CartRom(u16),
    CartHeader(u16),
    CartRam(u16),
    CartRomOtherBank(u16),
    Gfx(u16),
    ExternalRam(u16),
    Ram(u16),
    RamShadow(u16),
    SpriteInformation(u16),
    Io(u8),
    Stack(u8),
}

pub fn map_address(virtual_address: u16) -> Address {
    match virtual_address {
        CART_RAM_START ... CART_RAM_END => Address::CartRam(virtual_address - CART_RAM_START),
        CART_ROM_BANK_ZERO_START...CART_ROM_BANK_ZERO_END => {
            Address::CartRom(virtual_address - CART_ROM_BANK_ZERO_START)
        }
        GFX_RAM_START...GFX_RAM_END => Address::Gfx(virtual_address - GFX_RAM_START),
        RAM_START...RAM_END => Address::Ram(virtual_address - RAM_START),
        _ => panic!("Address {:#X} outside valid memory.", virtual_address),
    }
}

pub fn map_address_unwrap(virtual_address: u16) -> u16 {
    match virtual_address {
        CART_RAM_START ... CART_RAM_END => virtual_address - CART_RAM_START,
        CART_ROM_BANK_ZERO_START...CART_ROM_BANK_ZERO_END => {
            virtual_address - CART_ROM_BANK_ZERO_START
        }
        GFX_RAM_START...GFX_RAM_END => virtual_address - GFX_RAM_START,
        RAM_START...RAM_END => virtual_address - RAM_START,
        _ => panic!("Address {:#X} outside valid memory.", virtual_address),
    }
}

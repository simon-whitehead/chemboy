
const CART_ROM_START: u16 = 0x0000;
const CART_ROM_END: u16 = 0x7FFF;

const BIOS_START: u16 = 0x0000;
const BIOS_END: u16 = 0x00FF;

const CART_HEADER_START: u16 = 0x0100;
const CART_HEADER_END: u16 = 0x014F;

const GFX_RAM_START: u16 = 0x8000;
const GFX_RAM_END: u16 = 0x9FFF;

const CART_RAM_START: u16 = 0xA000;
const CART_RAM_END: u16 = 0xBFFF;

const RAM_START: u16 = 0xC000;
const RAM_END: u16 = 0xDFFF;

const RAM_SHADOW_START: u16 = 0xE000;
const RAM_SHADOW_END: u16 = 0xFDFF;

const GFX_SPRITE_INFO_START: u16 = 0xFE00;
const GFX_SPRITE_INFO_END: u16 = 0xFE9F;

const UNUSED_MEMORY_START: u16 = 0xFEA0;
const UNUSED_MEMORY_END: u16 = 0xFEFF;

const IO_START: u16 = 0xFF00;
const IO_END: u16 = 0xFF7F;

const ZRAM_START: u16 = 0xFF80;
const ZRAM_END: u16 = 0xFFFE;

const INTERRUPT_ENABLE_REGISTER: u16 = 0xFFFF;

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
    Oam(u16),
    Unused(u16),
    Io(u16),
    ZRam(u16),
    InterruptEnableRegister(u16),
}

pub fn map_address(virtual_address: u16) -> Address {
    match virtual_address {
        CART_ROM_START...CART_ROM_END => Address::CartRom(virtual_address - CART_ROM_START),
        CART_RAM_START...CART_RAM_END => Address::CartRam(virtual_address - CART_RAM_START),
        GFX_RAM_START...GFX_RAM_END => Address::Gfx(virtual_address - GFX_RAM_START),
        RAM_START...RAM_END => Address::Ram(virtual_address - RAM_START),
        RAM_SHADOW_START...RAM_SHADOW_END => Address::RamShadow(virtual_address - RAM_SHADOW_START),
        IO_START...IO_END => Address::Io(virtual_address - IO_START),
        ZRAM_START...ZRAM_END => Address::ZRam(virtual_address - ZRAM_START),
        GFX_SPRITE_INFO_START...GFX_SPRITE_INFO_END => {
            Address::Oam(virtual_address - GFX_SPRITE_INFO_START)
        }
        UNUSED_MEMORY_START...UNUSED_MEMORY_END => {
            Address::Unused(virtual_address - UNUSED_MEMORY_START)
        }
        INTERRUPT_ENABLE_REGISTER => {
            Address::InterruptEnableRegister(virtual_address - INTERRUPT_ENABLE_REGISTER)
        }
        _ => panic!("Address {:#X} outside valid memory.", virtual_address),
    }
}

pub fn map_address_unwrap(virtual_address: u16) -> u16 {
    match virtual_address {
        CART_RAM_START...CART_RAM_END => virtual_address - CART_RAM_START,
        CART_ROM_START...CART_ROM_END => virtual_address - CART_ROM_START,
        GFX_RAM_START...GFX_RAM_END => virtual_address - GFX_RAM_START,
        RAM_START...RAM_END => virtual_address - RAM_START,
        RAM_SHADOW_START...RAM_SHADOW_END => virtual_address - RAM_SHADOW_START,
        IO_START...IO_END => (virtual_address - IO_START),
        ZRAM_START...ZRAM_END => (virtual_address - ZRAM_START),
        GFX_SPRITE_INFO_START...GFX_SPRITE_INFO_END => virtual_address - GFX_SPRITE_INFO_START,
        UNUSED_MEMORY_START...UNUSED_MEMORY_END => virtual_address - UNUSED_MEMORY_START,
        INTERRUPT_ENABLE_REGISTER => virtual_address - INTERRUPT_ENABLE_REGISTER,
        _ => panic!("Address {:#X} outside valid memory.", virtual_address),
    }
}

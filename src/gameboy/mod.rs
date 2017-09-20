#[macro_use]
mod macros;

mod cartridge;
mod cpu;
mod gameboy;
pub mod gfx;
mod interconnect;
mod irq;
mod joypad;
mod mbc;
mod memory;
mod memory_map;
mod opcode;
mod registers;
mod timer;


pub use self::cartridge::{Cartridge, CartridgeDetails};
pub use self::cpu::{Cpu, CpuSpeed};
pub use gameboy::gfx::Frame;
pub use self::gameboy::GameBoy;
// pub use ::gameboy::gfx::Gpu;
pub use self::joypad::{Joypad, JoypadButton};
pub use self::memory::Memory;
pub use self::interconnect::Interconnect;
pub use self::irq::{Irq, Interrupt};
pub use self::timer::Timer;

pub use self::macros::*;

pub const CPU_FREQUENCY: usize = 0x400000; // 4,194,304
pub const MAX_CPU_CYCLES: usize = CPU_FREQUENCY / 60; // 60hz, our target refresh rate/fps
pub const MAX_DIV_REG_CYCLES: usize = MAX_CPU_CYCLES / 0x10;

pub const SCREEN_WIDTH: u32 = 0xA0;
pub const SCREEN_HEIGHT: u32 = 0x90;
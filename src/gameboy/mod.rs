#[macro_use]
mod macros;

mod cpu;
mod frame;
mod gameboy;
mod gpu;
mod interconnect;
mod irq;
mod memory;
mod memory_map;
mod registers;
mod timer;

pub mod cartridge;
pub mod opcode;

pub use self::cartridge::{Cartridge, CartridgeDetails};
pub use self::cpu::Cpu;
pub use self::frame::Frame;
pub use self::gameboy::GameBoy;
pub use self::gpu::Gpu;
pub use self::memory::Memory;
pub use self::interconnect::Interconnect;
pub use self::irq::{Irq, Interrupt};
pub use self::timer::Timer;

pub use self::macros::*;


pub const CPU_FREQUENCY: usize = 0x400000; // 4,194,304
pub const MAX_CPU_CYCLES: usize = CPU_FREQUENCY / 60; // 60hz, our target refresh rate/fps
pub const MAX_DIV_REG_CYCLES: usize = MAX_CPU_CYCLES / 0x10;

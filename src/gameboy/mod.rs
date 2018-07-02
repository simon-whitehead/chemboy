#[macro_use]
mod macros;

mod cartridge;
mod cpu;
mod disassembler;
pub mod debugger;
mod gameboy;
pub mod gfx;
mod interconnect;
mod irq;
mod joypad;
mod mbc;
mod memory;
mod memory_map;
pub mod opcodes;
mod registers;
mod serial;
mod timer;
pub mod ui;

pub use self::cartridge::{Cartridge, CartridgeDetails};
pub use self::cpu::{Cpu, CpuSpeed};
pub use self::disassembler::disassemble;
pub use self::gfx::Frame;
pub use self::gameboy::GameBoy;
pub use self::joypad::{Joypad, JoypadButton};
pub use self::memory::Memory;
pub use self::interconnect::Interconnect;
pub use self::irq::{Interrupt, Irq};
pub use self::timer::Timer;
pub use self::ui::ui::Ui;

pub use self::macros::*;

pub const CPU_FREQUENCY: usize = 0x400000; // 4,194,304
pub const MAX_CPU_CYCLES: usize = CPU_FREQUENCY / 60; // 60hz, our target refresh rate/fps
pub const MAX_DIV_REG_CYCLES: usize = MAX_CPU_CYCLES / 0x10;

pub const SCREEN_WIDTH: usize = 0xA0;
pub const SCREEN_HEIGHT: usize = 0x90;

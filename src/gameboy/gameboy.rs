use gameboy::cartridge::{Cartridge, CartridgeDetails};
use gameboy::cpu;
use ::gameboy::gfx::Frame;
use gameboy::joypad::JoypadButton;
use gameboy::interconnect::Interconnect;

pub struct GameBoy {
    pub cpu: cpu::Cpu,
    pub interconnect: Interconnect,
}

impl GameBoy {
    pub fn new(gameboy_color: bool, cart: Cartridge, disable_boot_rom: bool) -> GameBoy {
        let mut gb = GameBoy {
            cpu: cpu::Cpu::new(gameboy_color),
            interconnect: Interconnect::with_cart(cart, disable_boot_rom),
        };
        gb.reset();
        gb
    }

    pub fn run(&mut self) -> Result<(), String> {
        self.cpu.cycle(&mut self.interconnect)
    }

    pub fn reset(&mut self) {
        self.interconnect.reset();
        self.cpu.reset(&mut self.interconnect);
    }

    pub fn cart_details(&self) -> &CartridgeDetails {
        self.interconnect.cart_details()
    }

    pub fn request_frame(&self) -> &Frame {
        self.interconnect.request_frame()
    }

    pub fn press(&mut self, button: JoypadButton) {
        self.interconnect.press(button);
    }

    pub fn unpress(&mut self, button: JoypadButton) {
        self.interconnect.unpress(button);
    }

    pub fn set_speed(&mut self, speed: cpu::CpuSpeed) {
        self.cpu.speed = speed;
    }
}

use gameboy::cartridge::{Cartridge, CartridgeDetails};
use gameboy::cpu;
use gameboy::frame::Frame;
use gameboy::interconnect::Interconnect;

pub struct GameBoy {
    pub cpu: cpu::Cpu,
    pub interconnect: Interconnect,
}

impl GameBoy {
    pub fn new(gameboy_color: bool, cart: Cartridge) -> GameBoy {
        let mut gb = GameBoy {
            cpu: cpu::Cpu::new(gameboy_color),
            interconnect: Interconnect::with_cart(cart),
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

    pub fn cart_details(&self) -> CartridgeDetails {
        self.interconnect.cart_details()
    }

    pub fn request_frame(&self) -> &Frame {
        self.interconnect.request_frame()
    }
}

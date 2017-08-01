use gameboy::cartridge::{Cartridge, CartridgeDetails};
use gameboy::cpu;
use gameboy::interconnect::Interconnect;

pub struct GameBoy {
    cpu: cpu::Cpu,
    interconnect: Interconnect,
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

    pub fn run(&mut self) -> bool {
        self.cpu.cycle(&mut self.interconnect);

        return true;
    }

    pub fn reset(&mut self) {
        self.cpu.reset(&mut self.interconnect);
    }

    pub fn cart_details(&self) -> CartridgeDetails {
        self.interconnect.cart_details()
    }
}

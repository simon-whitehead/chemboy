use gameboy::cartridge::{Cartridge, CartridgeDetails};
use gameboy::cpu;
use gameboy::frame::Frame;
use gameboy::interconnect::Interconnect;

static NINTENDO_LOGO: [u8; 0x30] = [0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73,
                                    0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D, 0x00, 0x08, 0x11, 0x1F,
                                    0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD,
                                    0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC,
                                    0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E];

pub struct GameBoy {
    pub cpu: cpu::Cpu,
    pub interconnect: Interconnect,
}

impl GameBoy {
    pub fn new(gameboy_color: bool, cart: Cartridge, is_bootrom: bool) -> GameBoy {
        let mut gb = GameBoy {
            cpu: cpu::Cpu::new(gameboy_color),
            interconnect: Interconnect::with_cart(cart),
        };
        gb.reset();
        if is_bootrom {
            gb.cpu.registers.pc = 0x00;
            gb.interconnect.write_bytes(0x104, &NINTENDO_LOGO);
        }
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

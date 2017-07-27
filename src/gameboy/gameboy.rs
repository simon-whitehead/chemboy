use ::gameboy::cpu;
use ::gameboy::interconnect::Interconnect;

pub struct GameBoy {
    cpu: cpu::Cpu,
    interconnect: Interconnect,
}

impl GameBoy {
    pub fn new(gameboy_color: bool, rom: Vec<u8>) -> GameBoy {
        GameBoy {
            cpu: cpu::Cpu::new(gameboy_color),
            interconnect: Interconnect::with_rom(rom.into_boxed_slice()),
        }
    }

    pub fn run(&mut self) -> bool {
        self.cpu.step(&mut self.interconnect);

        return true;
    }
}

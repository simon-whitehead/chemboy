use super::cpu;

pub struct GameBoy {
    cpu: cpu::Cpu,
}

impl GameBoy {
    pub fn new(gameboy_color: bool, rom: Vec<u8>) -> GameBoy {
        GameBoy { cpu: cpu::Cpu::new(gameboy_color, rom) }
    }

    pub fn run(&mut self) -> bool {
        self.cpu.step();

        return true;
    }
}

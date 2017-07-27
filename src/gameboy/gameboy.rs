use ::gameboy::cpu;
use ::gameboy::interconnect::Interconnect;

pub struct GameBoy {
    pub game_title: String,

    cpu: cpu::Cpu,
    interconnect: Interconnect,
}

impl GameBoy {
    pub fn new(gameboy_color: bool, rom: Vec<u8>) -> GameBoy {
        let mut gb = GameBoy {
            cpu: cpu::Cpu::new(gameboy_color),
            interconnect: Interconnect::with_rom(rom.into_boxed_slice()),
            game_title: "Unknown".into()
        };
        gb.reset();
        gb
    }

    pub fn run(&mut self) -> bool {
        self.cpu.step(&mut self.interconnect);

        return true;
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.parse_game_title();
    }

    fn parse_game_title(&mut self) {
        let game_title_bytes = self.interconnect.read_bytes(0x134..0x142);
        self.game_title = String::from_utf8_lossy(&game_title_bytes).into();
    }
}

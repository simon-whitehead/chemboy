pub struct Registers {
    A: u8,
    B: u8,
    C: u8,
    D: u8,
    E: u8,
    F: u8,
    H: u8,
    L: u8,

    PC: u16,
    SP: u16,
}

impl Registers {
    pub fn new(gameboy_color: bool) -> Registers {
        // Taken from http://www.devrs.com/gb/files/gbspec.txt
        let a = match gameboy_color {
            true => 0x11,
            false => 0x01,
        };

        Registers {
            A: a,
            F: 0xB0,
            B: 0x00,
            C: 0x13,
            D: 0x00,
            E: 0xD8,
            H: 0x01,
            L: 0x4D,

            PC: 0x00,
            SP: 0xFFFE,
        }
    }
}

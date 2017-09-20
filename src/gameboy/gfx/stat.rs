use gameboy::gfx::Gpu;

pub struct GpuStat {
    pub coincidence_interrupt_enabled: bool,
    pub OAM_interrupt_enabled: bool,
    pub VBlank_interrupt_enabled: bool,
    pub HBlank_interrupt_enabled: bool,
}

impl GpuStat {
    pub fn new() -> GpuStat {
        GpuStat {
            coincidence_interrupt_enabled: false,
            OAM_interrupt_enabled: false,
            VBlank_interrupt_enabled: false,
            HBlank_interrupt_enabled: false,
        }
    }

    pub fn from_u8(b: u8) -> GpuStat {
        GpuStat {
            coincidence_interrupt_enabled: b & 0x40 == 0x40,
            OAM_interrupt_enabled: b & 0x20 == 0x20,
            VBlank_interrupt_enabled: b & 0x10 == 0x10,
            HBlank_interrupt_enabled: b & 0x08 == 0x08,
        }
    }

    pub fn to_u8(&self, gpu: &Gpu) -> u8 {
        (if self.coincidence_interrupt_enabled {
            0x40
        } else {
            0
        }) | (if self.OAM_interrupt_enabled { 0x20 } else { 0 }) |
        (if self.VBlank_interrupt_enabled {
            0x10
        } else {
            0
        }) |
        (if self.HBlank_interrupt_enabled {
            0x08
        } else {
            0
        }) | (if gpu.ly == gpu.lyc { 0x04 } else { 0 }) | gpu.mode.to_u8()
    }
}

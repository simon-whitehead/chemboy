#[derive(Eq, PartialEq, Clone)]
pub enum GpuMode {
    HBlank,
    VBlank,
    VRAM,
    OAM,
}

impl GpuMode {
    pub fn to_u8(&self) -> u8 {
        match *self {
            GpuMode::HBlank => 0x00,
            GpuMode::VBlank => 0x01,
            GpuMode::VRAM => 0x02,
            GpuMode::OAM => 0x03,
        }
    }
}

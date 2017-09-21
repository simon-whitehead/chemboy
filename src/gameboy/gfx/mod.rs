
mod color;
mod frame;
mod gpu;
mod mode;
mod sprite;
mod stat;
mod tile_render_options;

pub use self::color::Color;
pub use self::gpu::Gpu;
pub use self::frame::Frame;
pub use self::mode::GpuMode;
pub use self::sprite::SpriteShape;
pub use self::stat::GpuStat;

use self::tile_render_options::{TileRenderOptions, TileRenderType};

pub const VRAM_SIZE: usize = 0x4000;
pub const SPRITE_DATA_SIZE: usize = 0xA0;
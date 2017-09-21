
pub enum TileRenderType {
    Background,
    Window
}

pub struct TileRenderOptions {
    pub render_type: TileRenderType,
    pub map_addr: usize,
    pub tile_base_addr: usize,
    pub line: usize
}

impl TileRenderOptions {
    pub fn new(render_type: TileRenderType, line: usize, map_addr: usize, tile_base_addr: usize) -> TileRenderOptions {
        TileRenderOptions {
            render_type: render_type,
            line: line,
            map_addr: map_addr,
            tile_base_addr: tile_base_addr
        }
    }
}
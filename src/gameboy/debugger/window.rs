use gfx_device_gl::Factory;
use piston_window::*;

pub struct DebuggerWindow {
    piston_window: PistonWindow,
}

impl DebuggerWindow {
    pub fn new() -> DebuggerWindow {

        let opengl = OpenGL::V3_2;
        let mut window: PistonWindow = WindowSettings::new("chemboy debugger", [320, 144])
            .exit_on_esc(true)
            .opengl(opengl)
            .build()
            .unwrap();
        let window = window.position((10, 10));

        DebuggerWindow { piston_window: window }
    }

    pub fn set_pos<P>(&mut self, pos: P)
        where P: Into<(i32, i32)>
    {
        self.piston_window.show();
        self.piston_window.set_position(pos.into());
    }
}
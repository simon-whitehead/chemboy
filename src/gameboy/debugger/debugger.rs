
use gameboy::debugger::window::DebuggerWindow;

pub struct Debugger {
    pub window: DebuggerWindow,
}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger { window: DebuggerWindow::new() }
    }
}
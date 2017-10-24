
use gameboy::ui::theme::Theme;

pub enum UIEvent {
    None,
    ThemeSwitched(Theme)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
}

impl MouseButton {
    pub fn from_sdl(button: sdl3::mouse::MouseButton) -> Self {
        match button {
            sdl3::mouse::MouseButton::Left => Self::Left,
            sdl3::mouse::MouseButton::Right => Self::Right,
            sdl3::mouse::MouseButton::Middle => Self::Middle,
            sdl3::mouse::MouseButton::X1 => Self::X1,
            sdl3::mouse::MouseButton::X2 => Self::X2,
            _ => Self::Left,
        }
    }
}
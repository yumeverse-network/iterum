#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
}

impl MouseButton {
    pub fn from_winit(button: winit::event::MouseButton) -> Self {
        match button {
            winit::event::MouseButton::Left => Self::Left,
            winit::event::MouseButton::Right => Self::Right,
            winit::event::MouseButton::Middle => Self::Middle,

            winit::event::MouseButton::Other(code) => match code {
                8 => Self::X1,
                9 => Self::X2,
                _ => Self::Left,
            }

            winit::event::MouseButton::Back => Self::X1,
            winit::event::MouseButton::Forward => Self::X2,

            _ => Self::Left,
        }
    }
}
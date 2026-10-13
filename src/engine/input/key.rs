#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Unknown,

    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    Num0, Num1, Num2, Num3, Num4,
    Num5, Num6, Num7, Num8, Num9,

    Escape,
    Enter,
    Tab,
    Backspace,
    Space,

    Left,
    Right,
    Up,
    Down,

    Shift,
    Ctrl,
    Alt,

    F1, F2, F3, F4, F5, F6,
    F7, F8, F9, F10, F11, F12,
}

impl Key {
    pub fn from_scancode(
        scancode: winit::keyboard::KeyCode,
    ) -> Option<Self> {
        use winit::keyboard::KeyCode::*;
        
        Some(match scancode {
            KeyA => Self::A,
            KeyB => Self::B,
            KeyC => Self::C,
            KeyD => Self::D,
            KeyE => Self::E,
            KeyF => Self::F,
            KeyG => Self::G,
            KeyH => Self::H,
            KeyI => Self::I,
            KeyJ => Self::J,
            KeyK => Self::K,
            KeyL => Self::L,
            KeyM => Self::M,
            KeyN => Self::N,
            KeyO => Self::O,
            KeyP => Self::P,
            KeyQ => Self::Q,
            KeyR => Self::R,
            KeyS => Self::S,
            KeyT => Self::T,
            KeyU => Self::U,
            KeyV => Self::V,
            KeyW => Self::W,
            KeyX => Self::X,
            KeyY => Self::Y,
            KeyZ => Self::Z,
            
            Digit0 => Self::Num0,
            Digit1 => Self::Num1,
            Digit2 => Self::Num2,
            Digit3 => Self::Num3,
            Digit4 => Self::Num4,
            Digit5 => Self::Num5,
            Digit6 => Self::Num6,
            Digit7 => Self::Num7,
            Digit8 => Self::Num8,
            Digit9 => Self::Num9,
            
            Escape => Self::Escape,
            Enter => Self::Enter,
            Tab => Self::Tab,
            Backspace => Self::Backspace,
            Space => Self::Space,

            ArrowLeft => Self::Left,
            ArrowRight => Self::Right,
            ArrowUp => Self::Up,
            ArrowDown => Self::Down,

            winit::keyboard::KeyCode::ShiftLeft
            | winit::keyboard::KeyCode::ShiftRight => Self::Shift,
            
            winit::keyboard::KeyCode::ControlLeft
            | winit::keyboard::KeyCode::ControlRight => Self::Ctrl,
            
            winit::keyboard::KeyCode::AltLeft
            | winit::keyboard::KeyCode::AltRight => Self::Alt,
            
            F1 => Self::F1,
            F2 => Self::F2,
            F3 => Self::F3,
            F4 => Self::F4,
            F5 => Self::F5,
            F6 => Self::F6,
            F7 => Self::F7,
            F8 => Self::F8,
            F9 => Self::F9,
            F10 => Self::F10,
            F11 => Self::F11,
            F12 => Self::F12,

            _ => return None,
        })
    }
}
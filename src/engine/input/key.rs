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
        scancode: sdl3::keyboard::Scancode,
    ) -> Option<Self> {
        use sdl3::keyboard::Scancode::*;

        Some(match scancode {
            A => Self::A,
            B => Self::B,
            C => Self::C,
            D => Self::D,
            E => Self::E,
            F => Self::F,
            G => Self::G,
            H => Self::H,
            I => Self::I,
            J => Self::J,
            K => Self::K,
            L => Self::L,
            M => Self::M,
            N => Self::N,
            O => Self::O,
            P => Self::P,
            Q => Self::Q,
            R => Self::R,
            S => Self::S,
            T => Self::T,
            U => Self::U,
            V => Self::V,
            W => Self::W,
            X => Self::X,
            Y => Self::Y,
            Z => Self::Z,

            Num0 => Self::Num0,
            Num1 => Self::Num1,
            Num2 => Self::Num2,
            Num3 => Self::Num3,
            Num4 => Self::Num4,
            Num5 => Self::Num5,
            Num6 => Self::Num6,
            Num7 => Self::Num7,
            Num8 => Self::Num8,
            Num9 => Self::Num9,

            Escape => Self::Escape,
            Return => Self::Enter,
            Tab => Self::Tab,
            Backspace => Self::Backspace,
            Space => Self::Space,

            Left => Self::Left,
            Right => Self::Right,
            Up => Self::Up,
            Down => Self::Down,

            LShift | RShift => Self::Shift,
            LCtrl | RCtrl => Self::Ctrl,
            LAlt | RAlt => Self::Alt,

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
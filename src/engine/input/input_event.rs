use super::{
    ControllerAxis,
    ControllerButton,
    ControllerId,
    Key,
    MouseButton,
};

#[derive(Debug, Clone)]
pub enum InputEvent {
    Key {
        key: Key,
        pressed: bool,
        repeat: bool,
    },

    MouseButton {
        button: MouseButton,
        pressed: bool,
    },

    MouseMotion {
        position: [f32; 2],
        delta: [f32; 2],
    },

    ControllerButton {
        controller: ControllerId,
        button: ControllerButton,
        pressed: bool,
    },

    ControllerAxis {
        controller: ControllerId,
        axis: ControllerAxis,
        value: f32,
    },

    Touch {
        id: u64,
        position: [f32; 2],
        pressed: bool,
    },
}
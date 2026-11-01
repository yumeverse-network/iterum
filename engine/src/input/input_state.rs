use std::collections::{HashMap, HashSet};

use super::{
    ControllerAxis,
    ControllerButton,
    ControllerId,
    Key,
    MouseButton,
};

#[derive(Debug, Clone)]
pub struct InputState {
    keys: HashSet<Key>,
    mouse_buttons: HashSet<MouseButton>,

    mouse_position: [f32; 2],
    mouse_delta: [f32; 2],

    controller_buttons:
        HashMap<ControllerId, HashSet<ControllerButton>>,

    controller_axes:
        HashMap<(ControllerId, ControllerAxis), f32>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            keys: HashSet::new(),
            mouse_buttons: HashSet::new(),

            mouse_position: [0.0, 0.0],
            mouse_delta: [0.0, 0.0],

            controller_buttons: HashMap::new(),
            controller_axes: HashMap::new(),
        }
    }
}

impl InputState {
    pub fn key_pressed(&self, key: Key) -> bool {
        self.keys.contains(&key)
    }

    pub fn mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons.contains(&button)
    }

    pub fn mouse_position(&self) -> [f32; 2] {
        self.mouse_position
    }

    pub fn mouse_delta(&self) -> [f32; 2] {
        self.mouse_delta
    }

    pub fn controller_button_pressed(
        &self,
        controller: ControllerId,
        button: ControllerButton,
    ) -> bool {
        self.controller_buttons
            .get(&controller)
            .is_some_and(|buttons| buttons.contains(&button))
    }

    pub fn controller_axis(
        &self,
        controller: ControllerId,
        axis: ControllerAxis,
    ) -> f32 {
        self.controller_axes
            .get(&(controller, axis))
            .copied()
            .unwrap_or(0.0)
    }

    pub(crate) fn set_key(
        &mut self,
        key: Key,
        pressed: bool,
    ) {
        if pressed {
            self.keys.insert(key);
        } else {
            self.keys.remove(&key);
        }
    }

    pub(crate) fn set_mouse_button(
        &mut self,
        button: MouseButton,
        pressed: bool,
    ) {
        if pressed {
            self.mouse_buttons.insert(button);
        } else {
            self.mouse_buttons.remove(&button);
        }
    }

    pub(crate) fn set_mouse_motion(
        &mut self,
        position: [f32; 2],
        delta: [f32; 2],
    ) {
        self.mouse_position = position;
        self.mouse_delta = delta;
    }

    pub(crate) fn set_controller_button(
        &mut self,
        controller: ControllerId,
        button: ControllerButton,
        pressed: bool,
    ) {
        let buttons = self
            .controller_buttons
            .entry(controller)
            .or_default();

        if pressed {
            buttons.insert(button);
        } else {
            buttons.remove(&button);
        }
    }

    pub(crate) fn set_controller_axis(
        &mut self,
        controller: ControllerId,
        axis: ControllerAxis,
        value: f32,
    ) {
        self.controller_axes
            .insert((controller, axis), value);
    }

    pub(crate) fn reset_frame_data(&mut self) {
        self.mouse_delta = [0.0, 0.0];
    }
}
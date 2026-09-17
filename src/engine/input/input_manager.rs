use super::{
    ActionMap,
    Binding,
    ControllerAxis,
    ControllerButton,
    ControllerId,
    InputEvent,
    InputState,
    Key,
    MouseButton,
};

pub struct InputManager {
    current: InputState,
    previous: InputState,

    events: Vec<InputEvent>,

    pub actions: ActionMap,
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            current: InputState::default(),
            previous: InputState::default(),

            events: Vec::new(),

            actions: ActionMap::default(),
        }
    }

    pub fn begin_frame(&mut self) {
        self.previous = self.current.clone();
        self.current.reset_frame_data();
        self.events.clear();
    }

    pub fn process_event(&mut self, event: InputEvent) {
        match &event {
            InputEvent::Key {
                key,
                pressed,
                ..
            } => {
                self.current.set_key(*key, *pressed);
            }

            InputEvent::MouseButton {
                button,
                pressed,
            } => {
                self.current
                    .set_mouse_button(*button, *pressed);
            }

            InputEvent::MouseMotion {
                position,
                delta,
            } => {
                self.current
                    .set_mouse_motion(*position, *delta);
            }

            InputEvent::ControllerButton {
                controller,
                button,
                pressed,
            } => {
                self.current.set_controller_button(
                    *controller,
                    *button,
                    *pressed,
                );
            }

            InputEvent::ControllerAxis {
                controller,
                axis,
                value,
            } => {
                self.current.set_controller_axis(
                    *controller,
                    *axis,
                    *value,
                );
            }

            InputEvent::Touch { .. } => {}
        }

        self.events.push(event);
    }

    // =========================
    // Keyboard
    // =========================

    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.current.key_pressed(key)
    }

    pub fn is_key_just_pressed(&self, key: Key) -> bool {
        self.current.key_pressed(key)
            && !self.previous.key_pressed(key)
    }

    pub fn is_key_just_released(&self, key: Key) -> bool {
        !self.current.key_pressed(key)
            && self.previous.key_pressed(key)
    }

    // =========================
    // Mouse
    // =========================

    pub fn is_mouse_button_pressed(
        &self,
        button: MouseButton,
    ) -> bool {
        self.current.mouse_button_pressed(button)
    }

    pub fn is_mouse_button_just_pressed(
        &self,
        button: MouseButton,
    ) -> bool {
        self.current.mouse_button_pressed(button)
            && !self.previous.mouse_button_pressed(button)
    }

    pub fn is_mouse_button_just_released(
        &self,
        button: MouseButton,
    ) -> bool {
        !self.current.mouse_button_pressed(button)
            && self.previous.mouse_button_pressed(button)
    }

    pub fn mouse_position(&self) -> [f32; 2] {
        self.current.mouse_position()
    }

    pub fn mouse_delta(&self) -> [f32; 2] {
        self.current.mouse_delta()
    }

    // =========================
    // Controller
    // =========================

    pub fn is_controller_button_pressed(
        &self,
        controller: ControllerId,
        button: ControllerButton,
    ) -> bool {
        self.current
            .controller_button_pressed(controller, button)
    }

    pub fn controller_axis(
        &self,
        controller: ControllerId,
        axis: ControllerAxis,
    ) -> f32 {
        self.current.controller_axis(controller, axis)
    }

    // =========================
    // Actions
    // =========================

    pub fn is_action_pressed(
        &self,
        name: &str,
    ) -> bool {
        let Some(action) = self.actions.get(name) else {
            return false;
        };

        action.bindings.iter().any(|binding| {
            self.binding_pressed(binding)
        })
    }

    pub fn is_action_just_pressed(
        &self,
        name: &str,
    ) -> bool {
        let Some(action) = self.actions.get(name) else {
            return false;
        };

        action.bindings.iter().any(|binding| {
            self.binding_just_pressed(binding)
        })
    }

    pub fn is_action_just_released(
        &self,
        name: &str,
    ) -> bool {
        let Some(action) = self.actions.get(name) else {
            return false;
        };

        action.bindings.iter().any(|binding| {
            self.binding_just_released(binding)
        })
    }

    fn binding_pressed(&self, binding: &Binding) -> bool {
        match binding {
            Binding::Key(key) =>
                self.is_key_pressed(*key),

            Binding::MouseButton(button) =>
                self.is_mouse_button_pressed(*button),

            // Add controller logic here.
            Binding::ControllerButton(_) => false,

            Binding::ControllerAxis {
                axis,
                deadzone,
            } => {
                self.controller_axis(
                    ControllerId(0),
                    *axis,
                )
                .abs() > *deadzone
            }
        }
    }

    fn binding_just_pressed(
        &self,
        binding: &Binding,
    ) -> bool {
        match binding {
            Binding::Key(key) =>
                self.is_key_just_pressed(*key),

            Binding::MouseButton(button) =>
                self.is_mouse_button_just_pressed(*button),

            Binding::ControllerButton(_) => false,

            Binding::ControllerAxis { .. } => false,
        }
    }

    fn binding_just_released(
        &self,
        binding: &Binding,
    ) -> bool {
        match binding {
            Binding::Key(key) =>
                self.is_key_just_released(*key),

            Binding::MouseButton(button) =>
                self.is_mouse_button_just_released(*button),

            Binding::ControllerButton(_) => false,

            Binding::ControllerAxis { .. } => false,
        }
    }

    pub fn events(&self) -> &[InputEvent] {
        &self.events
    }
}

/*use super::{
    action::{ActionMap, Binding},
    controller::{ControllerAxis, ControllerButton, ControllerId},
    input_event::InputEvent,
    input_state::InputState,
    key::Key,
    mouse::MouseButton,
};

use std::{collections::HashSet, iter::Scan};

pub use sdl3::keyboard::Scancode;

pub struct InputAPI {
    pub keys_held: HashSet<Scancode>,
}

impl InputAPI {
    pub fn new() -> Self {
        Self {
            keys_held: HashSet::new(),
        }
    }
    
    pub fn is_key_pressed(&self, scancode: Scancode) -> bool {
        self.keys_held.contains(&scancode)
    }

    pub fn is_key_just_pressed(&self, scancode: Scancode) -> bool {
        todo!("INPUT. JUST PRESSED");
        false
    }
}*/
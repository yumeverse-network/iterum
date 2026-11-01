use std::collections::HashMap;

use super::{
    controller::{ControllerAxis, ControllerButton},
    key::Key,
    mouse::MouseButton,
};

#[derive(Debug, Clone)]
pub enum Binding {
    Key(Key),

    MouseButton(MouseButton),

    ControllerButton(ControllerButton),

    ControllerAxis {
        axis: ControllerAxis,
        deadzone: f32,
    },
}

#[derive(Debug, Clone)]
pub struct Action {
    pub bindings: Vec<Binding>,
}

impl Action {
    pub fn new(bindings: Vec<Binding>) -> Self {
        Self { bindings }
    }
}

#[derive(Debug, Default)]
pub struct ActionMap {
    actions: HashMap<String, Action>,
}

impl ActionMap {
    pub fn bind(
        &mut self,
        name: impl Into<String>,
        action: Action,
    ) {
        self.actions.insert(name.into(), action);
    }

    pub fn get(&self, name: &str) -> Option<&Action> {
        self.actions.get(name)
    }
}
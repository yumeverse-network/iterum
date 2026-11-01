#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControllerButton {
    South,
    East,
    West,
    North,

    Back,
    Start,

    LeftStick,
    RightStick,

    LeftShoulder,
    RightShoulder,

    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControllerAxis {
    LeftX,
    LeftY,
    RightX,
    RightY,
    LeftTrigger,
    RightTrigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ControllerId(pub u32);
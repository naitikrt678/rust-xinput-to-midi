use std::collections::HashSet;

use windows::Win32::UI::Input::XboxController::{
    XInputGetState, XINPUT_STATE,
};

// XInput button bitmask constants (u16)
const BTN_DPAD_UP: u16 = 0x0001;
const BTN_DPAD_DOWN: u16 = 0x0002;
const BTN_DPAD_LEFT: u16 = 0x0004;
const BTN_DPAD_RIGHT: u16 = 0x0008;
const BTN_START: u16 = 0x0010;
const BTN_BACK: u16 = 0x0020;
const BTN_LEFT_THUMB: u16 = 0x0040;
const BTN_RIGHT_THUMB: u16 = 0x0080;
const BTN_LEFT_SHOULDER: u16 = 0x0100;
const BTN_RIGHT_SHOULDER: u16 = 0x0200;
const BTN_A: u16 = 0x1000;
const BTN_B: u16 = 0x2000;
const BTN_X: u16 = 0x4000;
const BTN_Y: u16 = 0x8000;

/// Threshold for treating analog triggers as digital buttons (0–255 range).
const TRIGGER_THRESHOLD: u8 = 64;

/// All controller buttons supported by this application.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ControllerButton {
    A,
    B,
    X,
    Y,
    LB,
    RB,
    Back,
    Start,
    LStick,
    RStick,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    LT,
    RT,
}

impl ControllerButton {
    /// Human-friendly display name shown in the UI.
    pub fn display_name(&self) -> &'static str {
        match self {
            ControllerButton::A => "A",
            ControllerButton::B => "B",
            ControllerButton::X => "X",
            ControllerButton::Y => "Y",
            ControllerButton::LB => "LB  (Left Bumper)",
            ControllerButton::RB => "RB  (Right Bumper)",
            ControllerButton::Back => "Back",
            ControllerButton::Start => "Start",
            ControllerButton::LStick => "LS  (Left Stick Click)",
            ControllerButton::RStick => "RS  (Right Stick Click)",
            ControllerButton::DPadUp => "D-Pad  ↑",
            ControllerButton::DPadDown => "D-Pad  ↓",
            ControllerButton::DPadLeft => "D-Pad  ←",
            ControllerButton::DPadRight => "D-Pad  →",
            ControllerButton::LT => "LT  (Left Trigger digital)",
            ControllerButton::RT => "RT  (Right Trigger digital)",
        }
    }
}

/// Ordered list of all buttons for the mapping table.
pub const ALL_BUTTONS: &[ControllerButton] = &[
    ControllerButton::A,
    ControllerButton::B,
    ControllerButton::X,
    ControllerButton::Y,
    ControllerButton::LB,
    ControllerButton::RB,
    ControllerButton::Back,
    ControllerButton::Start,
    ControllerButton::LStick,
    ControllerButton::RStick,
    ControllerButton::DPadUp,
    ControllerButton::DPadDown,
    ControllerButton::DPadLeft,
    ControllerButton::DPadRight,
    ControllerButton::LT,
    ControllerButton::RT,
];

/// Poll the first connected XInput controller (ports 0-3).
///
/// Returns `(is_connected, set_of_currently_pressed_buttons)`.
pub fn poll_xinput() -> (bool, HashSet<ControllerButton>) {
    let mut state = XINPUT_STATE::default();

    // Try each of the four possible controller ports
    for port in 0u32..4 {
        let result = unsafe { XInputGetState(port, &mut state) };
        // ERROR_SUCCESS == 0
        if result == 0 {
            let pressed = decode_buttons(&state);
            return (true, pressed);
        }
    }

    (false, HashSet::new())
}

/// Decode a raw XINPUT_STATE into a set of pressed ControllerButton values.
fn decode_buttons(state: &XINPUT_STATE) -> HashSet<ControllerButton> {
    let mut pressed = HashSet::new();
    // .0 extracts the inner u16 from the XINPUT_GAMEPAD_BUTTON_FLAGS newtype
    let buttons: u16 = state.Gamepad.wButtons.0;

    macro_rules! check {
        ($flag:expr, $variant:expr) => {
            if buttons & $flag != 0 {
                pressed.insert($variant);
            }
        };
    }

    check!(BTN_A, ControllerButton::A);
    check!(BTN_B, ControllerButton::B);
    check!(BTN_X, ControllerButton::X);
    check!(BTN_Y, ControllerButton::Y);
    check!(BTN_LEFT_SHOULDER, ControllerButton::LB);
    check!(BTN_RIGHT_SHOULDER, ControllerButton::RB);
    check!(BTN_BACK, ControllerButton::Back);
    check!(BTN_START, ControllerButton::Start);
    check!(BTN_LEFT_THUMB, ControllerButton::LStick);
    check!(BTN_RIGHT_THUMB, ControllerButton::RStick);
    check!(BTN_DPAD_UP, ControllerButton::DPadUp);
    check!(BTN_DPAD_DOWN, ControllerButton::DPadDown);
    check!(BTN_DPAD_LEFT, ControllerButton::DPadLeft);
    check!(BTN_DPAD_RIGHT, ControllerButton::DPadRight);

    // Treat triggers as digital buttons above a threshold
    if state.Gamepad.bLeftTrigger >= TRIGGER_THRESHOLD {
        pressed.insert(ControllerButton::LT);
    }
    if state.Gamepad.bRightTrigger >= TRIGGER_THRESHOLD {
        pressed.insert(ControllerButton::RT);
    }

    pressed
}

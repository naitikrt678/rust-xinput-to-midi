use std::collections::HashMap;
use std::path::Path;

use crate::input::ControllerButton;

/// Serialization-friendly mapping type (String key for JSON compatibility)
type SerialMap = HashMap<String, u8>;

/// Save button→note mappings to a JSON file.
pub fn save_mappings(
    mappings: &HashMap<ControllerButton, u8>,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let serial: SerialMap = mappings
        .iter()
        .map(|(btn, &note)| (button_to_key(btn), note))
        .collect();

    let json = serde_json::to_string_pretty(&serial)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Load button→note mappings from a JSON file.
pub fn load_mappings(
    path: &Path,
) -> Result<HashMap<ControllerButton, u8>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let serial: SerialMap = serde_json::from_str(&content)?;

    let mut mappings = HashMap::new();
    for (key, note) in serial {
        if let Some(btn) = key_to_button(&key) {
            if note <= 127 {
                mappings.insert(btn, note);
            }
        }
    }

    Ok(mappings)
}

/// Convert a ControllerButton to its JSON key string.
fn button_to_key(btn: &ControllerButton) -> String {
    use ControllerButton::*;
    match btn {
        A => "A",
        B => "B",
        X => "X",
        Y => "Y",
        LB => "LB",
        RB => "RB",
        Back => "Back",
        Start => "Start",
        LStick => "LStick",
        RStick => "RStick",
        DPadUp => "DPadUp",
        DPadDown => "DPadDown",
        DPadLeft => "DPadLeft",
        DPadRight => "DPadRight",
        LT => "LT",
        RT => "RT",
    }
    .to_string()
}

/// Parse a JSON key string back to a ControllerButton.
fn key_to_button(key: &str) -> Option<ControllerButton> {
    use ControllerButton::*;
    match key {
        "A" => Some(A),
        "B" => Some(B),
        "X" => Some(X),
        "Y" => Some(Y),
        "LB" => Some(LB),
        "RB" => Some(RB),
        "Back" => Some(Back),
        "Start" => Some(Start),
        "LStick" => Some(LStick),
        "RStick" => Some(RStick),
        "DPadUp" => Some(DPadUp),
        "DPadDown" => Some(DPadDown),
        "DPadLeft" => Some(DPadLeft),
        "DPadRight" => Some(DPadRight),
        "LT" => Some(LT),
        "RT" => Some(RT),
        _ => None,
    }
}

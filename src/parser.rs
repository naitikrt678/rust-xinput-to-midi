use std::collections::HashMap;

/// Parse a MIDI note-name map from plain text content.
///
/// Format expected (one entry per line):
/// ```text
/// # comment lines are ignored
/// 36 Kick
/// 37 Snare - Hit
/// ```
///
/// Returns a map of MIDI note number → display name.
pub fn parse_note_map(content: &str) -> HashMap<u8, String> {
    let mut map = HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip blank lines and comment lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // First token must be a valid u8 MIDI note number
        let mut parts = trimmed.splitn(2, char::is_whitespace);
        let Some(note_str) = parts.next() else {
            continue;
        };
        let Ok(note) = note_str.parse::<u8>() else {
            continue;
        };
        if note > 127 {
            continue;
        }

        // Remainder of the line is the display name (trimmed)
        let name = parts.next().unwrap_or("").trim().to_string();
        if !name.is_empty() {
            map.insert(note, name);
        }
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parse() {
        let input = "# MIDI note/CC name map\n36 Kick\n37 Snare - Hit\n\n# comment\n49 Hat - Closed\n";
        let map = parse_note_map(input);
        assert_eq!(map.get(&36), Some(&"Kick".to_string()));
        assert_eq!(map.get(&37), Some(&"Snare - Hit".to_string()));
        assert_eq!(map.get(&49), Some(&"Hat - Closed".to_string()));
    }

    #[test]
    fn test_ignores_bad_lines() {
        let input = "not_a_number Kick\n200 Out of range\n36 Kick";
        let map = parse_note_map(input);
        assert!(map.get(&36).is_some());
        assert_eq!(map.len(), 1);
    }
}

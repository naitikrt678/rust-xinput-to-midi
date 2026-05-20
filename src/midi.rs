use windows::{
    core::HSTRING,
    Devices::{
        Enumeration::DeviceInformation,
        Midi::{IMidiOutPort, MidiNoteOffMessage, MidiNoteOnMessage, MidiOutPort},
    },
};

/// MIDI channel 10 (zero-indexed = 9) — standard GM percussion channel.
const DRUM_CHANNEL: u8 = 9;
const VELOCITY: u8 = 127;

/// A live MIDI output connection using Windows.Devices.Midi (WinRT).
/// Replaces midir — this API sees virtual ports (loopMIDI, etc.) that
/// the legacy WinMM API often misses.
pub struct MidiOutputConnection {
    port: IMidiOutPort,
}

/// Enumerate MIDI output devices via the WinRT device stack.
/// Returns (display_name, device_id) pairs.
pub fn list_midi_outputs() -> Vec<(String, String)> {
    (|| -> windows::core::Result<Vec<(String, String)>> {
        let selector = MidiOutPort::GetDeviceSelector()?;
        let devices = DeviceInformation::FindAllAsyncAqsFilter(&selector)?.get()?;
        let count = devices.Size()?;
        eprintln!("[midi] WinRT found {} port(s):", count);
        let mut out = Vec::new();
        for i in 0..count {
            let dev = devices.GetAt(i)?;
            let name = dev.Name()?.to_string();
            let id = dev.Id()?.to_string();
            eprintln!("[midi]   {:?}", name);
            out.push((name, id));
        }
        Ok(out)
    })()
    .unwrap_or_else(|e| {
        eprintln!("[midi] ERROR enumerating: {:?}", e);
        Vec::new()
    })
}

/// Open a MIDI output connection by WinRT device ID.
pub fn open_midi_output(
    device_id: &str,
) -> Result<MidiOutputConnection, Box<dyn std::error::Error>> {
    let id = HSTRING::from(device_id);
    let port = MidiOutPort::FromIdAsync(&id)?.get()?;
    Ok(MidiOutputConnection { port })
}

/// Send MIDI Note-On on channel 10, velocity 127.
#[inline]
pub fn send_note_on(conn: &mut MidiOutputConnection, note: u8) {
    if let Ok(msg) = MidiNoteOnMessage::CreateMidiNoteOnMessage(DRUM_CHANNEL, note, VELOCITY) {
        let _ = conn.port.SendMessage(&msg);
    }
}

/// Send MIDI Note-Off on channel 10, velocity 0.
#[inline]
pub fn send_note_off(conn: &mut MidiOutputConnection, note: u8) {
    if let Ok(msg) = MidiNoteOffMessage::CreateMidiNoteOffMessage(DRUM_CHANNEL, note, 0) {
        let _ = conn.port.SendMessage(&msg);
    }
}

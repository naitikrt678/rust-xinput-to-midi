use std::collections::{HashMap, HashSet};

use crate::config;
use crate::input::{poll_xinput, ControllerButton, ALL_BUTTONS};
use crate::midi::{list_midi_outputs, open_midi_output, send_note_off, send_note_on, MidiOutputConnection};
use crate::parser::parse_note_map;
use rfd::FileDialog;

/// Default MIDI note assignments for each button.
fn default_mappings() -> HashMap<ControllerButton, u8> {
    use ControllerButton::*;
    let mut m = HashMap::new();
    m.insert(A, 36);
    m.insert(B, 37);
    m.insert(X, 38);
    m.insert(Y, 39);
    m.insert(LB, 46);
    m.insert(RB, 49);
    m.insert(Back, 42);
    m.insert(Start, 44);
    m.insert(LStick, 48);
    m.insert(RStick, 45);
    m.insert(DPadUp, 50);
    m.insert(DPadDown, 41);
    m.insert(DPadLeft, 43);
    m.insert(DPadRight, 47);
    m.insert(LT, 54);
    m.insert(RT, 55);
    m
}

pub struct ControllerMidiApp {
    /// Persistent mappings: button → MIDI note
    mappings: HashMap<ControllerButton, u8>,
    /// Optional human-readable note names
    note_names: HashMap<u8, String>,
    /// Available MIDI output port names (display)
    midi_ports: Vec<String>,
    /// WinRT device IDs corresponding to midi_ports
    midi_device_ids: Vec<String>,
    /// Index into midi_ports of currently selected port
    selected_midi_port: usize,
    /// Active MIDI output connection (None if none opened yet)
    midi_conn: Option<MidiOutputConnection>,
    /// Button states from last frame (for edge detection)
    prev_buttons: HashSet<ControllerButton>,
    /// Is a controller detected this frame?
    controller_connected: bool,
    /// Editable text fields for the mapping table (button → text buffer)
    note_edit_buffers: HashMap<ControllerButton, String>,
    /// Status / error message shown at the bottom
    status_msg: String,
}

impl ControllerMidiApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mappings = default_mappings();
        let note_edit_buffers = mappings
            .iter()
            .map(|(btn, &note)| (btn.clone(), note.to_string()))
            .collect();

        let port_pairs = list_midi_outputs();
        let midi_ports: Vec<String> = port_pairs.iter().map(|(n, _)| n.clone()).collect();
        let midi_device_ids: Vec<String> = port_pairs.iter().map(|(_, id)| id.clone()).collect();
        let selected_midi_port = 0;

        let mut app = Self {
            mappings,
            note_names: HashMap::new(),
            midi_ports,
            midi_device_ids,
            selected_midi_port,
            midi_conn: None,
            prev_buttons: HashSet::new(),
            controller_connected: false,
            note_edit_buffers,
            status_msg: String::from("Ready. Select a MIDI output and connect a controller."),
        };

        // Auto-connect to first available port
        if !app.midi_ports.is_empty() {
            app.connect_midi(0);
        }

        app
    }

    fn connect_midi(&mut self, port_index: usize) {
        if port_index >= self.midi_device_ids.len() {
            return;
        }
        let device_id = self.midi_device_ids[port_index].clone();
        match open_midi_output(&device_id) {
            Ok(conn) => {
                self.midi_conn = Some(conn);
                self.selected_midi_port = port_index;
                self.status_msg =
                    format!("Connected to: {}", self.midi_ports[port_index]);
            }
            Err(e) => {
                self.midi_conn = None;
                self.status_msg = format!("MIDI error: {}", e);
            }
        }
    }

    fn refresh_midi_ports(&mut self) {
        let port_pairs = list_midi_outputs();
        self.midi_ports = port_pairs.iter().map(|(n, _)| n.clone()).collect();
        self.midi_device_ids = port_pairs.iter().map(|(_, id)| id.clone()).collect();
        self.midi_conn = None;
        self.status_msg = String::from("MIDI port list refreshed.");
        if !self.midi_ports.is_empty() {
            self.connect_midi(0);
        }
    }

    fn import_note_map(&mut self) {
        let file = FileDialog::new()
            .add_filter("Text file", &["txt"])
            .add_filter("All files", &["*"])
            .set_title("Import Note Name Map")
            .pick_file();

        if let Some(path) = file {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    self.note_names = parse_note_map(&content);
                    self.status_msg = format!(
                        "Imported {} note names from {}",
                        self.note_names.len(),
                        path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("file")
                    );
                }
                Err(e) => {
                    self.status_msg = format!("Failed to read file: {}", e);
                }
            }
        }
    }

    fn save_config(&self) {
        let file = FileDialog::new()
            .add_filter("JSON", &["json"])
            .set_file_name("controller_midi_config.json")
            .set_title("Save Mapping Config")
            .save_file();

        if let Some(path) = file {
            match config::save_mappings(&self.mappings, &path) {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("Save error: {}", e);
                }
            }
        }
    }

    fn load_config(&mut self) {
        let file = FileDialog::new()
            .add_filter("JSON", &["json"])
            .set_title("Load Mapping Config")
            .pick_file();

        if let Some(path) = file {
            match config::load_mappings(&path) {
                Ok(loaded) => {
                    // Merge loaded mappings into current
                    for (btn, note) in &loaded {
                        self.mappings.insert(btn.clone(), *note);
                        self.note_edit_buffers
                            .insert(btn.clone(), note.to_string());
                    }
                    self.status_msg = format!(
                        "Loaded {} mappings from {}",
                        loaded.len(),
                        path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("file")
                    );
                }
                Err(e) => {
                    self.status_msg = format!("Load error: {}", e);
                }
            }
        }
    }

    /// Poll XInput, detect edge transitions, emit MIDI.
    fn poll_and_emit(&mut self) {
        let (connected, current_buttons) = poll_xinput();
        self.controller_connected = connected;

        if !connected {
            self.prev_buttons.clear();
            return;
        }

        // Collect events first (avoids borrow conflicts on self fields)
        let mut note_ons: Vec<u8> = Vec::new();
        let mut note_offs: Vec<u8> = Vec::new();

        for btn in &current_buttons {
            if !self.prev_buttons.contains(btn) {
                if let Some(&note) = self.mappings.get(btn) {
                    note_ons.push(note);
                }
            }
        }
        for btn in &self.prev_buttons {
            if !current_buttons.contains(btn) {
                if let Some(&note) = self.mappings.get(btn) {
                    note_offs.push(note);
                }
            }
        }

        // Now send all events
        if let Some(conn) = &mut self.midi_conn {
            for note in note_ons {
                send_note_on(conn, note);
            }
            for note in note_offs {
                send_note_off(conn, note);
            }
        }

        self.prev_buttons = current_buttons;
    }
}

impl eframe::App for ControllerMidiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll input every frame (egui drives this via continuous repaint)
        self.poll_and_emit();
        ctx.request_repaint();

        // Apply dark visuals
        ctx.set_visuals(egui::Visuals::dark());

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(6.0);

            // ── Header ────────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.heading("🎮  Controller → MIDI");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Controller status pill
                    let (label, color) = if self.controller_connected {
                        ("● Controller: Connected", egui::Color32::from_rgb(80, 200, 120))
                    } else {
                        ("○ Controller: Disconnected", egui::Color32::from_rgb(200, 80, 80))
                    };
                    ui.label(egui::RichText::new(label).color(color).small());
                });
            });

            ui.separator();
            ui.add_space(4.0);

            // ── MIDI Output row ────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("MIDI Output:");

                let port_label = if self.midi_ports.is_empty() {
                    "No MIDI ports found".to_string()
                } else {
                    self.midi_ports
                        .get(self.selected_midi_port)
                        .cloned()
                        .unwrap_or_default()
                };

                let port_names: Vec<String> = self.midi_ports.clone();
                let mut clicked_port: Option<usize> = None;
                egui::ComboBox::from_id_source("midi_port_combo")
                    .selected_text(&port_label)
                    .width(260.0)
                    .show_ui(ui, |ui| {
                        for (i, name) in port_names.iter().enumerate() {
                            let resp = ui.selectable_value(
                                &mut self.selected_midi_port,
                                i,
                                name,
                            );
                            if resp.clicked() {
                                clicked_port = Some(i);
                            }
                        }
                    });
                if let Some(idx) = clicked_port {
                    self.connect_midi(idx);
                }

                if ui.small_button("⟳ Refresh").clicked() {
                    self.refresh_midi_ports();
                }

                // MIDI connection indicator
                let (dot, dot_color) = if self.midi_conn.is_some() {
                    ("●", egui::Color32::from_rgb(80, 200, 120))
                } else {
                    ("○", egui::Color32::from_rgb(200, 80, 80))
                };
                ui.label(egui::RichText::new(dot).color(dot_color));
            });

            ui.add_space(6.0);

            // ── Action buttons ─────────────────────────────────────────
            ui.horizontal(|ui| {
                if ui.button("📂  Import Note Map").clicked() {
                    self.import_note_map();
                }
                if ui.button("💾  Save Config").clicked() {
                    self.save_config();
                }
                if ui.button("📁  Load Config").clicked() {
                    self.load_config();
                }
            });

            ui.add_space(8.0);
            ui.separator();

            // ── Mapping Table ──────────────────────────────────────────
            ui.label(
                egui::RichText::new("Button Mappings")
                    .strong()
                    .size(13.0),
            );
            ui.add_space(4.0);

            // Table header
            egui::Grid::new("mapping_header")
                .num_columns(3)
                .min_col_width(160.0)
                .spacing([8.0, 2.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Button").strong().size(11.0));
                    ui.label(egui::RichText::new("MIDI Note #").strong().size(11.0));
                    ui.label(egui::RichText::new("Note Name").strong().size(11.0));
                    ui.end_row();
                });

            ui.separator();

            // Scrollable table body
            egui::ScrollArea::vertical()
                .max_height(340.0)
                .show(ui, |ui| {
                    egui::Grid::new("mapping_table")
                        .num_columns(3)
                        .min_col_width(160.0)
                        .striped(true)
                        .spacing([8.0, 4.0])
                        .show(ui, |ui| {
                            let buttons: Vec<ControllerButton> =
                                ALL_BUTTONS.iter().cloned().collect();

                            for btn in buttons {
                                // Column 1: button name
                                ui.label(btn.display_name());

                                // Column 2: editable note number
                                // Pre-compute the default so we don't borrow
                                // self twice in the entry().or_insert_with() call.
                                let default_note = self
                                    .mappings
                                    .get(&btn)
                                    .map(|n| n.to_string())
                                    .unwrap_or_default();
                                let buf = self
                                    .note_edit_buffers
                                    .entry(btn.clone())
                                    .or_insert(default_note);

                                let response = ui.add(
                                    egui::TextEdit::singleline(buf)
                                        .desired_width(70.0)
                                        .hint_text("0-127"),
                                );

                                if response.lost_focus() || response.changed() {
                                    if let Ok(n) = buf.trim().parse::<u8>() {
                                        if n <= 127 {
                                            self.mappings.insert(btn.clone(), n);
                                        }
                                    }
                                }

                                // Column 3: resolved note name
                                let note_name = self
                                    .mappings
                                    .get(&btn)
                                    .and_then(|n| self.note_names.get(n))
                                    .cloned()
                                    .unwrap_or_else(|| "—".to_string());

                                ui.label(
                                    egui::RichText::new(&note_name)
                                        .color(egui::Color32::from_rgb(160, 200, 255))
                                        .size(12.0),
                                );

                                ui.end_row();
                            }
                        });
                });

            ui.add_space(6.0);
            ui.separator();

            // ── Status bar ─────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(&self.status_msg)
                        .color(egui::Color32::from_rgb(180, 180, 180))
                        .size(11.0),
                );
            });
        });
    }
}

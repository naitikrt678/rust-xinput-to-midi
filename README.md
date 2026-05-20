# 🎮 Controller → MIDI

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg?logo=rust&style=flat-square)](https://www.rust-lang.org)
[![OS](https://img.shields.io/badge/OS-Windows%2010%2F11-blue.svg?logo=windows&style=flat-square)](#)
[![API](https://img.shields.io/badge/API-WinRT%20%2F%20XInput-ff69b4.svg?style=flat-square)](#)
[![MIDI](https://img.shields.io/badge/MIDI-Channel%2010-purple.svg?style=flat-square)](#)

A high-performance, lightweight Windows desktop application built in Rust that converts XInput gamepad button presses into low-latency MIDI note events. Specially designed and optimized for low-latency virtual drum controller mapping.

---

## ✨ Features

- **⚡ Zero-Latency Performance** – Direct polling of XInput controllers and native WinRT MIDI connection.
- **🔌 Auto-Detection** – Automatically scans and connects to Xbox/XInput controllers (polls all 4 slots).
- **🚫 Spam Prevention** – Edge-detection algorithm ensures a single `Note-On` event is sent when pressed, and a single `Note-Off` is sent on release (no repeat spam while holding buttons).
- **🎹 GM Percussion Standard** – Outputs to MIDI channel 10 with fixed maximum velocity (127) as per GM specs.
- **🎛️ Digital Triggers** – Left and Right triggers (LT/RT) are treated as digital buttons with a customizable analog threshold (triggers at >25% pull).
- **📂 Note Mapping** – Load simple, human-readable plain-text note name maps (`.txt`) for easy visual assignments.
- **💾 Configuration Management** – Save and load your customized button-to-note maps as light `.json` config files.
- **🔄 Live Remapping** – Update bindings on-the-fly inside the clean, dark-themed user interface while the app is actively running.
- **🔊 Virtual/Hardware Compatible** – Works seamlessly with virtual MIDI drivers like `loopMIDI` and any external hardware MIDI outputs.

---

## 🚀 Quick Start & Usage

1. **Setup MIDI Port**: Start your virtual MIDI driver (e.g., **loopMIDI**) or connect your hardware MIDI device.
2. **Launch App**: Open `controller-midi.exe`.
3. **Configure Output**: Select your virtual/physical MIDI output port from the dropdown menu.
4. **Connect Controller**: Connect any standard Xbox or XInput gamepad. The connection indicator will light up **green**.
5. **(Optional) Load Map**: Click **Import Note Map** and choose a `.txt` mapping file (like `example_note_map.txt`) to assign names to notes.
6. **Customize**: Click any MIDI note number in the table to re-assign it.
7. **Play!**: Press buttons on the gamepad. You will see notes fire in real time with minimum latency!

---

## 🛠️ Build Requirements

The project compiles on Windows using the **GNU toolchain** via **MSYS2 UCRT64**. *No bulky Visual Studio installations or MSVC Build Tools are required.*

### 1. Install MSYS2 & GCC
1. Download and run the installer from [msys2.org](https://www.msys2.org).
2. Open the **MSYS2 UCRT64** terminal and install the compiler toolchain:
   ```bash
   pacman -S mingw-w64-ucrt-x86_64-gcc
   ```
3. Add the MSYS2 bin directory to your Windows system `PATH` (User or System environment variables):
   ```text
   C:\msys64\ucrt64\bin
   ```

### 2. Install Rust
1. Download the installer from [rustup.rs](https://rustup.rs) or run the command:
   ```powershell
   winget install Rustlang.Rustup
   ```
2. Open a new terminal and switch to the stable GNU toolchain:
   ```powershell
   rustup default stable-gnu
   ```
3. Verify your environment is ready:
   ```powershell
   rustc --version
   cargo --version
   gcc --version
   ```

### 3. Build & Run locally
- **Development build (faster compile, has console window for logs):**
  ```powershell
  cargo run
  ```
- **Release build (fully optimized):**
  ```powershell
  cargo build --release
  ```

---

## 📦 How to Build into a Standalone `.exe`

To package this application as a clean, polished Windows executable (`.exe`) suitable for distribution, follow these steps:

### Step 1: Hide the Console Window in Production
By default, GUI applications in Rust will launch a command prompt window in the background to show logs. To suppress this console window in your final build:

1. Open [src/main.rs](file:///c:/Users/Naitik%20Behera/Desktop/projects/controller%20to%20midi%20in%20rust/src/main.rs).
2. Uncomment the very first line of the file:
   ```rust
   #![windows_subsystem = "windows"]
   ```
   *(Keep it commented out during development if you want to inspect stdout/stderr logs!)*

### Step 2: Compile the Release Binary
Run the compilation with the `--release` flag. This invokes high-level optimizations (`opt-level = 3`), Link-Time Optimization (`lto = true`), and automatic debug symbol stripping (`strip = true` already configured in `Cargo.toml`):

```powershell
cargo build --release
```

Once the compilation completes, your standalone executable is generated at:
```text
.\target\release\controller-midi.exe
```
This binary is completely **self-contained**. You can copy it anywhere and distribute it without needing any other assets.

### Step 3: Add a Custom Application Icon (Optional but Highly Recommended!)
To give your final `.exe` a custom logo (instead of the default blank Windows executable icon):

1. **Prepare your Icon**: Create or acquire a `.ico` file (e.g., `icon.ico`) and place it in the root of your project directory.
2. **Add winres**: Open your [Cargo.toml](file:///c:/Users/Naitik%20Behera/Desktop/projects/controller%20to%20midi%20in%20rust/Cargo.toml) and add `winres` as a build dependency:
   ```toml
   [build-dependencies]
   winres = "0.1"
   ```
3. **Create a Build Script**: In your project's root folder, create a new file named `build.rs` and add the following content:
   ```rust
   fn main() {
       if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
           let mut res = winres::WindowsResource::new();
           res.set_icon("icon.ico");
           res.compile().unwrap();
       }
   }
   ```
4. Run `cargo build --release` again. The new executable will feature your custom icon!

---

## 🔍 File Formats

### Note Name Map Format (`.txt`)
A plain text file containing space-separated note numbers and labels:
```text
# Comment lines start with '#' and blank lines are ignored
36 Kick
37 Snare - Hit
38 Snare - Rim
49 Hat - Closed
```

### Config File Format (`.json`)
A simple JSON object mapping controller button identifiers to MIDI note numbers:
```json
{
  "A": 36,
  "B": 37,
  "X": 38,
  "Y": 39,
  "LB": 46,
  "RB": 49,
  "LT": 54,
  "RT": 55
}
```

---

## 🗂️ Project Structure

```text
controller-midi/
├── .cargo/               # Cargo configuration
├── json maps/            # Pre-configured MIDI mappings
├── src/
│   ├── main.rs           # Entry point and subsystem settings
│   ├── app.rs            # UI layout and interactive state
│   ├── midi.rs           # WinRT MIDI API connectivity
│   ├── input.rs          # XInput polling and edge-detection
│   ├── parser.rs         # Note Map plain text parser
│   └── config.rs         # JSON configurations load/save
├── Cargo.toml            # Project dependencies & compile profiles
├── Cargo.lock            # Exact dependency versions locked
├── README.md             # This guide
└── example_note_map.txt  # Sample map file for testing
```

---

## 🔧 Technical Details
- **MIDI Channel**: Channel 10 (percussion standard; `0x99` note-on, `0x89` note-off).
- **Triggers**: Left/Right triggers poll from 0 to 255. A digital threshold of **64** (25%) acts as the trigger line.
- **Polling Loop**: Repaints continuously driven by `eframe` and egui's continuous repaint requesting, ensuring latency is kept to an absolute minimum.

---

## 🩺 Troubleshooting

### ❌ `dlltool: could not create import library` / `Invalid bfd target`

- **Symptom**: Native compilation fails when installing dependencies or running cargo tasks.
- **Cause**: An older or alternative MinGW installation (like `C:\MinGW`) exists in your system `PATH` and is shadowing the MSYS2 GCC.
- **Fix**:
  1. Remove `C:\MinGW` (or similar paths) from your environment system `PATH`.
  2. Verify MSYS2's path (`C:\msys64\ucrt64\bin`) is the only compiler toolchain available.
  3. Restart your terminal session to apply changes.
  4. Run `where.exe gcc` and verify it displays `C:\msys64\ucrt64\bin\gcc.exe`.

<div align="center">
  <a href="https://github.com/jeffreytse/winsoftvol">
    <img alt="WinSoftVol — Software Volume Bridge" src="assets/logo.svg" width="600">
  </a>

  <p>🔊 Software volume control for USB audio devices on Windows.</p>

<br><h1>⚒️ WinSoftVol ⚒️</h1>

</div>

<p align="center">
  <a href="https://github.com/sponsors/jeffreytse">
    <img src="https://img.shields.io/static/v1?label=sponsor&message=%E2%9D%A4&logo=GitHub&link=&color=greygreen"
      alt="Donate (GitHub Sponsor)" />
  </a>

  <a href="https://github.com/jeffreytse/winsoftvol/releases">
    <img src="https://img.shields.io/github/v/release/jeffreytse/winsoftvol?color=brightgreen"
      alt="Release Version" />
  </a>

  <a href="https://opensource.org/licenses/MIT">
    <img src="https://img.shields.io/badge/License-MIT-brightgreen.svg"
      alt="License: MIT" />
  </a>

  <a href="https://img.shields.io/badge/platform-Windows-blue">
    <img src="https://img.shields.io/badge/platform-Windows-blue"
      alt="Platform: Windows" />
  </a>
</p>

<div align="center">
  <h4>
    <a href="#-why-winsoftvol">Why</a> |
    <a href="#-features">Features</a> |
    <a href="#-requirements">Requirements</a> |
    <a href="#-installation">Install</a> |
    <a href="#-usage">Usage</a> |
    <a href="#-platform-notes">Platforms</a> |
    <a href="#%EF%B8%8F-how-it-works">How It Works</a> |
    <a href="#-configuration">Config</a> |
    <a href="#%EF%B8%8F-building">Build</a> |
    <a href="#-license">License</a>
  </h4>
</div>

<div align="center">
  <sub>Built with ❤︎ by
  <a href="https://jeffreytse.net">jeffreytse</a> and
  <a href="https://github.com/jeffreytse/winsoftvol/graphs/contributors">contributors</a>
  </sub>
</div>

<br>

## 🤔 Why WinSoftVol?

You plug in a USB audio device — a DAC, a USB sound card, maybe a WONDOM UCM board — and everything seems fine. Music plays. Then you reach for the taskbar volume slider and drag it down. The number changes. The slider moves. Nothing happens. You press the mute key. Still nothing.

You dig through device properties, update drivers, try every Windows audio setting you can find. Nothing works. You resign yourself to controlling volume from within each application individually, tab by tab, window by window — a small but persistent frustration every single time.

What is actually happening: Windows stores the volume change in the audio endpoint and expects the hardware to act on it. Your device, like many USB Audio Class devices without a Feature Unit in their descriptor, has no hardware volume control to speak of — so the driver silently ignores the request. The per-application session volumes are never touched.

WinSoftVol fixes this. It sits in the system tray, watches the endpoint for changes, and immediately propagates them as software volume to every running audio session. The taskbar slider works. Keyboard keys work. Mute works. It just works — the way it always should have.

## ✨ Features

- 🎚️ Intercepts system volume changes (taskbar slider, keyboard volume keys, mute button) and applies them to all audio sessions as software volume.
- 🖥️ System tray icon — unobtrusive, always available, zero UI overhead.
- 🔌 Automatic re-plug detection — reconnecting the USB device restores control within one second, confirmed by a toast notification.
- 🚀 Autostart on Windows startup via the registry Run key, toggled from the tray menu.
- 🔇 Force software volume mode — disables hardware volume on capable devices, routing all attenuation through the session mixer for cleaner, step-free control.
- 🔒 Volume cap — configurable ceiling on maximum output; presets defined in `config.toml` and selectable from the tray menu.
- 🌙 Night mode — automatically lowers the volume cap on a configurable schedule (e.g. 22:00–07:00); toggled from the tray menu without editing the config file.
- 🔈 Startup volume — optionally set a fixed volume level on every launch, regardless of what Windows last had; selectable from the tray menu.
- 📌 Device pinning — lock WinSoftVol to a specific audio device by friendly name; falls back to the default device with a notification if the pinned device is absent.
- ⚙️ Config file with hot-reload — persistent settings in `%APPDATA%\WinSoftVol\config.toml`; changes apply immediately without restarting.
- 🔔 Auto-update check — checks GitHub for new releases on startup; fires a clickable toast notification that opens the release page when a newer version is found.
- 🖱️ Scroll wheel on tray icon — adjust volume up/down per notch without touching the taskbar; step size configurable via `scroll_step_percent`.
- 🔕 Left-click tray icon — instantly toggle mute/unmute.
- 🔊 Dynamic tray icon — volume bar overlaid on the icon updates in real time; bar turns red when muted; tooltip shows current volume percentage and active cap.
- ⚠️ Exclusive mode detection — detects when a game or DAW bypasses the session mixer and notifies you why volume control stops working for that app.
- ℹ️ About dialog with version, build commit hash, build timestamp, links to the project homepage and GitHub Sponsors, and a download link when a newer version is available.
- 🦀 Written in Rust — small binary, no runtime, minimal resource usage.

## 💼 Requirements

- Windows 10 or later (x64)

## 📦 Installation

1. Download the latest `winsoftvol-vX.Y.Z-<hash>.exe` from the [Releases](https://github.com/jeffreytse/winsoftvol/releases) page.
2. Run it. A speaker icon appears in the system tray.
3. Optional: right-click the tray icon → **Start on Windows startup** to enable autostart.

No installer. No admin rights required. Single executable.

## 📚 Usage

Right-click the tray icon to open the menu.

| Menu Item                | Effect                                                                                                                        |
| ------------------------ | ----------------------------------------------------------------------------------------------------------------------------- |
| About WinSoftVol         | Shows version, build info, links, and a download link if a newer version is available                                         |
| Start on Windows startup | Toggles autostart (registry Run key)                                                                                          |
| Force software volume    | Routes all volume control through the session mixer; disables hardware attenuation on capable devices                         |
| Night mode               | Enables or disables the scheduled cap reduction; saved to config immediately                                                  |
| Max volume               | Sets a ceiling on output — presets are configurable in `config.toml`; the slider still covers 0–100% but is scaled to the cap |
| Startup volume           | Sets the volume applied on the next launch (Off = leave unchanged); saved to config immediately                               |
| Quit WinSoftVol          | Exits cleanly                                                                                                                 |

Tray icon also responds to direct interaction:

| Action            | Effect                                                    |
| ----------------- | --------------------------------------------------------- |
| Left-click        | Toggle mute / unmute                                      |
| Scroll wheel up   | Increase volume by `scroll_step_percent`% per notch (default 2%) |
| Scroll wheel down | Decrease volume by `scroll_step_percent`% per notch       |
| Hover             | Tooltip shows current volume % and active cap             |

Once running, use Windows volume controls normally:

| Control                      | Effect                                |
| ---------------------------- | ------------------------------------- |
| Taskbar volume slider        | Adjusts volume for all audio sessions |
| Keyboard volume up/down keys | Same                                  |
| Keyboard mute key            | Mutes / unmutes all audio sessions    |

## 🌍 Platform Notes

This problem is Windows-specific. On other platforms, the audio stack already handles software volume transparently:

- **Linux (PulseAudio / PipeWire)**: The audio server applies software volume during mixing before audio reaches the driver. Whether the hardware has a Feature Unit is irrelevant — volume is scaled in the server, not the hardware.
- **macOS (CoreAudio)**: The HAL applies software volume attenuation for devices that report no hardware volume capability. The system volume slider controls the HAL mix level, not a hardware register.
- **Windows**: Volume changes are written to the endpoint and the driver is expected to apply them in hardware. If the device has no Feature Unit, the driver ignores the change. Per-application session volumes are not updated unless something bridges them — which is what WinSoftVol does.

## ⚙️ How It Works

Windows exposes audio through the Core Audio API. Each device has an **endpoint volume** (`IAudioEndpointVolume`) and a set of per-application **session volumes** (`ISimpleAudioVolume`).

On normal hardware, Windows writes the endpoint volume and the driver applies it. On devices without a Feature Unit, the driver ignores the endpoint level entirely.

Windows audio output is a product of two levels:

```
output = session_volume (per-app mixer) × endpoint_volume (system slider) × audio_content
```

On normal hardware the driver applies both. On USB devices without a hardware Feature Unit the driver ignores the endpoint level entirely, making the system slider ineffective. WinSoftVol bridges the gap by moving attenuation into the session layer where software always applies it.

WinSoftVol:

1. Registers `IAudioEndpointVolumeCallback` on the default render device. Every time the endpoint level changes (slider, key press), `OnNotify` fires on the COM STA thread.
2. Inside `OnNotify`, reads each audio session's current volume via `IAudioSessionManager2` and scales it proportionally by `(new_level / old_level)` — preserving any per-app balance the user set in the Windows Volume Mixer.
3. Registers `IAudioSessionNotification` so applications started after WinSoftVol are initialised at the correct volume automatically.
4. Registers `IMMNotificationClient` to detect device plug/unplug events and reinitialise the bridge within one second, confirmed by a toast notification.
5. Polls `IAudioMeterInformation` once per second to detect when a game or DAW takes WASAPI exclusive mode (bypassing the session mixer) and notifies the user.

## 📝 Configuration

WinSoftVol reads `%APPDATA%\WinSoftVol\config.toml` on startup and reloads it automatically when the file changes — no restart required.

**Example `config.toml`:**

```toml
[general]
autostart            = false
cap_presets          = [100, 80, 60, 40]   # presets shown in tray Max volume submenu
scroll_step_percent  = 2                   # % per scroll notch (1–20)
startup_volume       = 50                  # set to 50% on launch; omit to leave unchanged
pin_device           = "Speakers (USB Audio Device)"  # omit to use Windows default device

# Night mode: lower cap automatically on a schedule
night_start          = "22:00"
night_end            = "07:00"
night_cap            = 40
night_enabled        = true

[default]
force_sw_volume = false
cap_percent     = 80

# Per-device overrides — key is the device friendly name (as shown in Windows Sound settings)
[device."Speakers (USB Audio Device)"]
force_sw_volume = true
cap_percent     = 60
```

| Key | Default | Description |
| --- | ------- | ----------- |
| `general.autostart` | `false` | Launch at Windows startup |
| `general.cap_presets` | `[100, 80, 60, 40]` | Volume cap presets shown in tray submenu (10–100, sorted descending) |
| `general.scroll_step_percent` | `2` | Volume change per scroll notch in % (1–20) |
| `general.startup_volume` | _(absent)_ | Volume to apply on each launch in % (0–100); omit to leave unchanged |
| `general.pin_device` | _(absent)_ | Friendly name of device to target; omit to use Windows default |
| `general.night_start` | _(absent)_ | Night mode start time (HH:MM); both `night_start` and `night_end` required |
| `general.night_end` | _(absent)_ | Night mode end time (HH:MM); wraps midnight (e.g. 22:00 → 07:00) |
| `general.night_cap` | `40` | Volume cap applied during night window (10–100) |
| `general.night_enabled` | `true` | Enable/disable the night schedule; toggled by tray "Night mode" item |
| `default.force_sw_volume` | `false` | Force software volume mode |
| `default.cap_percent` | `100` | Volume ceiling in % (10–100) |
| `device."<name>".*` | — | Per-device overrides (same keys as `[default]`); key is the device friendly name |

Per-device settings take precedence over `[default]` when the matching device is active (or pinned via `pin_device`). The device friendly name is the string shown in Windows Sound settings (e.g. "Speakers (USB Audio Device)").

## 🛠️ Building

Requires Rust (stable).

#### macOS — cross-compile to Windows x64

```sh
# First time only
make setup   # installs rustup target x86_64-pc-windows-gnu + mingw-w64

# Build versioned release binary
make         # produces dist/winsoftvol-v<version>-<hash>.exe
```

#### Windows — native build

```sh
cargo build --release
```

## 🔫 Contributing

Issues and Pull Requests are welcome. If you've never contributed to an open source project before, feel free to [open an issue](https://github.com/jeffreytse/winsoftvol/issues/new) describing the problem and we'll go from there.

## 🌈 License

This project is licensed under the [MIT license](https://opensource.org/licenses/mit-license.php) © Jeffrey Tse.

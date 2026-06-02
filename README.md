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
- 🔒 Volume cap — sets a ceiling on maximum output (100% / 80% / 60% / 40%) so the full slider range stays usable while preventing any app from blasting past the limit.
- 🖱️ Scroll wheel on tray icon — adjust volume up/down 2% per notch without touching the taskbar.
- 🔕 Left-click tray icon — instantly toggle mute/unmute.
- 🔊 Dynamic tray icon — volume bar overlaid on the icon updates in real time; bar turns red when muted.
- ⚠️ Exclusive mode detection — detects when a game or DAW bypasses the session mixer and notifies you why volume control stops working for that app.
- ℹ️ About dialog with version, build commit hash, and build timestamp.
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

| Menu Item                | Effect                                                                                                                   |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------ |
| About WinSoftVol         | Shows version, build info, and description                                                                               |
| Start on Windows startup | Toggles autostart (registry Run key)                                                                                     |
| Force software volume    | Routes all volume control through the session mixer; disables hardware attenuation on capable devices                    |
| Max volume               | Sets a ceiling on output: 100% / 80% / 60% / 40% of full scale — the slider still covers 0–100% but is scaled to the cap |
| Quit WinSoftVol          | Exits cleanly                                                                                                            |

Tray icon also responds to direct interaction:

| Action            | Effect                          |
| ----------------- | ------------------------------- |
| Left-click        | Toggle mute / unmute            |
| Scroll wheel up   | Increase volume by 2% per notch |
| Scroll wheel down | Decrease volume by 2% per notch |

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

# Sound Control

A Windows system tray app that makes the taskbar volume slider, keyboard volume keys, and mute button work correctly on USB audio devices that lack hardware volume control.

## Why This Exists

USB audio devices that implement the USB Audio Class without a Feature Unit — such as the **WONDOM UCM (PCM2706C)** — report no hardware volume capability to Windows. Windows still lets you drag the taskbar slider or press volume keys, and it stores the value in the endpoint, but it never sends the change to the hardware because there's nothing to send it to. The result: the slider moves, the number changes, nothing actually happens.

Sound Control bridges the gap. It watches the Windows audio endpoint for volume and mute changes and immediately propagates them to the software session volumes of every running application. This is the same mechanism that the volume mixer uses ("application volume"), so it works even when the hardware ignores the endpoint level.

## Features

- Intercepts system volume changes (taskbar slider, keyboard keys, mute button) and applies them to all audio sessions
- Tray icon with tooltip; right-click for the menu
- Main window with a volume slider and mute checkbox
- Automatic re-plug detection: reconnecting the USB device restores control within one second
- Optional autostart on Windows startup (written to the registry Run key)
- About dialog showing version, build commit, and build time

## Usage

Run `sound-control.exe`. A speaker icon appears in the system tray.

| Action | Effect |
|---|---|
| Right-click tray icon | Opens menu |
| Open Sound Control | Shows the volume control window |
| System volume slider / keys | Volume applied to all audio sessions |
| Mute button | Mutes all audio sessions |
| Start on Windows startup | Registers the exe in `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` |
| Quit Sound Control | Exits cleanly |

When the main window is open you can also drag the slider or toggle the Mute checkbox directly. Close the window to send it back to the tray; the app keeps running.

## Building

Requires Rust (stable) and a Windows cross-compile toolchain on macOS/Linux, or a native Windows Rust toolchain.

### macOS (cross-compile to Windows x64)

```sh
# First time only
make setup   # installs rustup target + mingw-w64

# Build
make         # release binary at target/x86_64-pc-windows-gnu/release/sound-control.exe

# Versioned dist package
make dist    # copies to dist/sound-control-v<version>-<hash>.exe
```

### Windows (native)

```sh
cargo build --release
```

## How It Works

Windows exposes audio through the Core Audio API. Each device has an **endpoint volume** (`IAudioEndpointVolume`) and a set of per-application **session volumes** (`ISimpleAudioVolume`).

On normal hardware, Windows writes the endpoint volume and the driver applies it. On devices without a Feature Unit, the driver ignores the endpoint level entirely.

Sound Control:

1. Registers `IAudioEndpointVolumeCallback` on the default render device. Every time the endpoint level changes (user moves the slider, presses a key), `OnNotify` fires on the STA thread.
2. Inside `OnNotify`, enumerates all active audio sessions via `IAudioSessionManager2` and sets each session's master volume and mute to match the endpoint level.
3. Registers `IAudioSessionNotification` so newly started applications are also picked up.
4. Watches for device plug/unplug via `IMMNotificationClient` and reinitialises the bridge within one second.

## License

MIT

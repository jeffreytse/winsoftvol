# Release v0.3.1

This release fixes a bug where scroll-wheel volume control would occasionally trigger while scrolling in other applications.

### 🐛 Bug Fixes

* **Scroll outside tray no longer changes volume:** The global mouse hook now verifies that the scroll event's cursor position falls within the system tray notification area before adjusting volume. Previously, if the mouse-leave event was missed (e.g. when a context menu stole mouse capture), the hook would process any scroll anywhere on screen — including webpages and other apps — until the cursor re-entered the tray icon (https://github.com/jeffreytse/winsoftvol/commit/f7e0f22)

---

**Full Changelog:** https://github.com/jeffreytse/winsoftvol/compare/v0.3.0...v0.3.1

---

# Release v0.3.0

This release adds scheduled night mode, startup volume, per-device pinning, configurable cap presets, configurable scroll step, and a live tray tooltip — all configurable from `config.toml` with hot-reload and most togglable directly from the tray menu.

### 🚀 Features

* **Night Mode:** Automatically lower the volume cap on a configurable time window (e.g. 22:00–07:00). Enabled/disabled from the tray "Night mode" toggle; saved to config immediately. Wraps midnight correctly (https://github.com/jeffreytse/winsoftvol/commit/b04432c)
* **Startup Volume:** Set a fixed volume level on every launch, regardless of what Windows last had. Selectable from the new "Startup volume" tray submenu (Off + cap presets) (https://github.com/jeffreytse/winsoftvol/commit/e8b6a46)
* **Device Pinning:** Lock WinSoftVol to a specific audio device by friendly name via `pin_device` in `[general]`. Falls back to the Windows default device with a toast notification when the pinned device is absent. Per-device `[device."..."]` config sections are now wired up and apply their cap/softvol settings when the matching device is active (https://github.com/jeffreytse/winsoftvol/commit/33d8394)
* **Configurable Cap Presets:** Replace the hardcoded 100/80/60/40 presets with `cap_presets` in `[general]`. Any sorted list of values in 10–100 is accepted; the tray submenu rebuilds from config on hot-reload (https://github.com/jeffreytse/winsoftvol/commit/661d8a4)
* **Configurable Scroll Step:** `scroll_step_percent` in `[general]` controls how much each scroll notch adjusts volume (default 2%, range 1–20%). Hot-reload applies the new step immediately (https://github.com/jeffreytse/winsoftvol/commit/d13197d)
* **Tray Tooltip:** Hovering the tray icon now shows the current volume percentage and active cap (e.g. `75% | cap: 80%`; `Muted | cap: 80%`) (https://github.com/jeffreytse/winsoftvol/commit/6f44c15)
* **DPI Awareness:** Application manifest updated to `PerMonitorV2` DPI awareness so the tray icon renders correctly on high-DPI displays (https://github.com/jeffreytse/winsoftvol/commit/ea7d4be)

### 🐛 Bug Fixes

* Volume cap now applies immediately to existing audio sessions when a preset is selected from the tray menu or when the config is hot-reloaded — previously, the new cap only took effect after the next volume change (https://github.com/jeffreytse/winsoftvol/commit/3f3d8b2)

### 📝 Documentation

* Update `README.md` to document all v0.3.0 features, expanded config table, and updated tray menu reference (https://github.com/jeffreytse/winsoftvol/commit/06566a9)

### 💅 Style & Formatting

* Apply `cargo fmt` formatting (https://github.com/jeffreytse/winsoftvol/commit/a9ee0f2)

---

**Full Changelog:** https://github.com/jeffreytse/winsoftvol/compare/v0.2.0...v0.3.0

---

# Release v0.2.0

This release significantly expands WinSoftVol with new tray interactions, a persistent config file with hot-reload, volume cap controls, device change notifications, exclusive mode detection, an upgraded About dialog, and automatic update detection.

### 🚀 Features

* **Volume Cap:** Add volume cap support — set a ceiling on maximum output (100% / 80% / 60% / 40%) from the tray menu (https://github.com/jeffreytse/winsoftvol/commit/52288f4d043f82f9df019883c26afbfad8bfded0)
* **Scroll Wheel:** Adjust volume ±2% per notch directly on the tray icon (https://github.com/jeffreytse/winsoftvol/commit/2ca5a6b801a3dccc42314c824c39521a1fccf0c7)
* **Left-click Mute:** Toggle mute/unmute with a single left-click on the tray icon (https://github.com/jeffreytse/winsoftvol/commit/da4a7ed8145e7daf83ddf75166c0f88b0046c702)
* **Dynamic Tray Icon:** Real-time volume bar overlay on the tray icon; bar turns red when muted (https://github.com/jeffreytse/winsoftvol/commit/e0338d17da39a5cd2e551dd954d743c5f8b93eb2)
* **Device Change Notification:** Toast notification confirms re-initialization after USB plug/unplug (https://github.com/jeffreytse/winsoftvol/commit/cadd0f901f491a7ec6b95bd9c7499157d1dac650)
* **Exclusive Mode Warning:** Detects when a game or DAW takes WASAPI exclusive mode and notifies the user (https://github.com/jeffreytse/winsoftvol/commit/a444ca414b7b4837af56331562f84c1dbf62c06e)
* **Config File:** Persistent settings in `%APPDATA%\WinSoftVol\config.toml` with hot-reload — changes apply without restarting; supports per-device overrides (https://github.com/jeffreytse/winsoftvol/commit/3e670284ce6a2f13f61f970504d93304eb85a740)
* **About Dialog:** Upgraded to TaskDialog with clickable links to the project homepage and GitHub Sponsors; shows a download link when a newer version is available (https://github.com/jeffreytse/winsoftvol/commit/e7c630a318772f4b6b665f6ab31b508f9bb89900)
* **Auto-update Check:** Background thread checks GitHub for new releases on startup. Fires a clickable toast notification (opens the release page directly) and surfaces a download link inside the About dialog when a newer version is found (https://github.com/jeffreytse/winsoftvol/commit/5db0ddce8c71fe822a8711bcdefdb1b849d3aa26)

### 🐛 Bug Fixes

* Resolve volume cap logic bugs (https://github.com/jeffreytse/winsoftvol/commit/81d5f2d07bfdaa6ebeef00e4b77d4bdba92db8bc)
* Volume cap menu items now correctly uncheck the previously selected preset (https://github.com/jeffreytse/winsoftvol/commit/85e7a35832adeb51d03c7d361fcf50c4fa0e1df1)
* Left-click tray icon now correctly opens the popup menu (https://github.com/jeffreytse/winsoftvol/commit/82b43613524b2499ab5a6736e2b9bb3f9f87d0f7)
* Resolve wrong volume applied when `GetMasterVolume` fails (https://github.com/jeffreytse/winsoftvol/commit/163296ea10f3545d3ceab8092d72d2dd5f7827ac)
* Resolve incorrect string length calculation in UTF-16 encoding (https://github.com/jeffreytse/winsoftvol/commit/c91e7be14cf12938e8887fb041a4c2ce28ddfde3)
* Clamp `cap_percent` to `[10, 100]` after TOML parse — a value of `0` previously silenced all audio (https://github.com/jeffreytse/winsoftvol/commit/83476a5f87fd54948892edb0fbe5f683153bbc01)
* Embed comctl32 v6 manifest — fixes "procedure entry point TaskDialogIndirect could not be located" crash on startup (https://github.com/jeffreytse/winsoftvol/commit/a57248e0bac9caf00e496b55be3d5f59e7ebbb79)

### 📝 Documentation

* Update `README.md` to reflect current features (https://github.com/jeffreytse/winsoftvol/commit/3edc774432846731bbf270423f8eb1ee3884b8c7)
* Add `LICENSE` file (https://github.com/jeffreytse/winsoftvol/commit/d8b9aa8836e1f943b92e5fd4322106b1459fcb02)
* Add `CONTRIBUTING.md` (https://github.com/jeffreytse/winsoftvol/commit/6215abc42e31223766f3e04bd5c820a41f856cbe)
* Document config file, hot-reload, and About dialog links in README (https://github.com/jeffreytse/winsoftvol/commit/48bdb85d96a39711a3e0fad1c9464781f27fbaf8)
* Document auto-update check feature in README (https://github.com/jeffreytse/winsoftvol/commit/06d752869ff367afb0f261e7315437694805bd09)

### 🧪 Tests & CI

* Expand test coverage across audio session management, config parsing, and tray presets (https://github.com/jeffreytse/winsoftvol/commit/4ecdd19c4fe1c7b5ee6191d757968335393ee0e6)

### 💅 Style & Formatting

* Resolve code formatting issues and align with styling guidelines (https://github.com/jeffreytse/winsoftvol/commit/f504ef22f91538a84cedd43de41018d5e47a6549, https://github.com/jeffreytse/winsoftvol/commit/a3a90b7bc59cc3058599d91701314425877f8851)

### 🔧 Build & Maintenance

* Add `make lint` command to check format and lint errors (https://github.com/jeffreytse/winsoftvol/commit/07369742a98360f7b284800bec3a745664528265)
* Add `make fix` command to quickly apply fmt and lint fixes (https://github.com/jeffreytse/winsoftvol/commit/271d5b7bc56a863722ac87a6bc1b021497828334)
* Update funding sources in `FUNDING.yml` (https://github.com/jeffreytse/winsoftvol/commit/ede4c2c)

---

**Full Changelog:** https://github.com/jeffreytse/winsoftvol/compare/v0.1.0...v0.2.0

---

# Release v0.1.0

The initial release of WinSoftVol — a system tray application that bridges software volume for USB audio devices on Windows that lack hardware volume control.

### 🚀 Features

* **Software Volume Bridge:** Registers `IAudioEndpointVolumeCallback` on the default render device; every endpoint volume change (taskbar slider, keyboard keys, mute button) is immediately propagated to all audio sessions as software volume (https://github.com/jeffreytse/winsoftvol/commit/b486650)
* **Per-app Balance Preservation:** Sessions are scaled proportionally by `(new_level / old_level)` so any per-app balance set in the Windows Volume Mixer is preserved across volume changes (https://github.com/jeffreytse/winsoftvol/commit/e1c9cce)
* **Force Software Volume Mode:** Disables hardware volume on capable devices, routing all attenuation through the session mixer; toggled from the tray menu (https://github.com/jeffreytse/winsoftvol/commit/6ea2254)
* **System Tray Icon:** Minimal tray presence with right-click menu: About, Start on Windows startup, Force software volume, Quit (https://github.com/jeffreytse/winsoftvol/commit/b486650)
* **Autostart:** Toggles the Windows registry Run key so WinSoftVol starts with Windows (https://github.com/jeffreytse/winsoftvol/commit/b486650)
* **About Dialog:** Shows version and build commit hash (https://github.com/jeffreytse/winsoftvol/commit/b486650)
* **Device Reconnect Detection:** Registers `IMMNotificationClient` and reinitialises the audio bridge within one second of a USB plug/unplug event (https://github.com/jeffreytse/winsoftvol/commit/b486650)

### 🧪 Tests & CI

* Add initial test coverage for audio session management (https://github.com/jeffreytse/winsoftvol/commit/1d6ee49)
* Add release workflow for cross-compiled Windows x64 binary (https://github.com/jeffreytse/winsoftvol/commit/d523fca)

### 📝 Documentation

* Add README with feature overview, usage instructions, platform notes, and build guide (https://github.com/jeffreytse/winsoftvol/commit/bdf0350)

---

**Full Changelog:** https://github.com/jeffreytse/winsoftvol/commits/v0.1.0

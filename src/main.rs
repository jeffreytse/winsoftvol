#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
mod about;
#[cfg(windows)]
mod audio;
#[cfg(windows)]
mod autostart;
mod config;
#[cfg(windows)]
mod notification;
mod tray;
#[cfg(windows)]
mod updater;

#[cfg(windows)]
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

#[cfg(windows)]
static CURSOR_OVER_TRAY: AtomicBool = AtomicBool::new(false);

// Accumulates scroll notches from the mouse hook; drained each message loop iteration.
#[cfg(windows)]
static PENDING_SCROLL: AtomicI32 = AtomicI32::new(0);

#[cfg(windows)]
unsafe extern "system" fn mouse_hook_proc(
    code: i32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, HHOOK, MSLLHOOKSTRUCT, WM_MOUSEWHEEL,
    };
    if code >= 0 && wparam.0 as u32 == WM_MOUSEWHEEL && CURSOR_OVER_TRAY.load(Ordering::Relaxed) {
        let data = &*(lparam.0 as *const MSLLHOOKSTRUCT);
        let delta = (data.mouseData >> 16) as i16;
        if delta > 0 {
            PENDING_SCROLL.fetch_add(1, Ordering::Relaxed);
        } else {
            PENDING_SCROLL.fetch_sub(1, Ordering::Relaxed);
        }
    }
    CallNextHookEx(HHOOK::default(), code, wparam, lparam)
}

#[cfg(windows)]
fn main() -> anyhow::Result<()> {
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)?;
    }
    let result = run();
    unsafe { CoUninitialize() };
    result
}

#[cfg(windows)]
fn run() -> anyhow::Result<()> {
    use std::sync::{atomic::AtomicU32, Arc, Mutex, RwLock};
    use windows::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, KillTimer, SetTimer, SetWindowsHookExW,
            TranslateMessage, UnhookWindowsHookEx, MSG, WH_MOUSE_LL,
        },
    };

    notification::register_aumid();

    let update_state: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    updater::spawn_update_checker(Arc::clone(&update_state));

    let initial_cfg = config::Config::load();
    let softvol_flag = Arc::new(AtomicBool::new(initial_cfg.default.force_sw_volume));
    let cap_flag = Arc::new(AtomicU32::new(initial_cfg.default.cap_percent));
    let tray_state = tray::build_tray(
        initial_cfg.general.autostart,
        initial_cfg.default.force_sw_volume,
        initial_cfg.default.cap_percent,
    )?;
    let cfg_state = Arc::new(RwLock::new(initial_cfg));

    // Track config file mtime to detect external edits for hot-reload
    let mut last_config_mtime: Option<std::time::SystemTime> =
        std::fs::metadata(config::Config::path())
            .ok()
            .and_then(|m| m.modified().ok());

    let watcher = audio::DeviceWatcher::new()?;
    let mut bridge: Option<audio::AudioBridge> =
        audio::AudioBridge::new(softvol_flag.clone(), cap_flag.clone()).ok();

    let hook = unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), None, 0)? };

    unsafe { SetTimer(HWND(0), 1, 1000, None) };

    let mut last_display: Option<(u32, bool)> = None;
    let mut exclusive_mode_active = false;
    let mut update_notified = false;
    let mut msg = MSG::default();
    loop {
        unsafe {
            if !GetMessageW(&mut msg, None, 0, 0).as_bool() {
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Drain scroll notches accumulated by the mouse hook
        let scroll = PENDING_SCROLL.swap(0, Ordering::Relaxed);
        if scroll != 0 {
            if let Some(ref b) = bridge {
                let _ = b.adjust_volume(scroll as f32 * 0.02);
            }
        }

        // Update tray icon when volume or mute state changes
        if let Some(ref b) = bridge {
            let (vol, muted) = b.current_volume();
            let pct = (vol * 100.0).round() as u32;
            let display = (pct, muted);
            if Some(display) != last_display {
                last_display = Some(display);
                if let Ok(icon) = tray::render_volume_icon(vol, muted) {
                    let _ = tray_state.update_icon(icon);
                }
            }
        }

        // Timer tick: exclusive mode check + config hot-reload
        if msg.message == windows::Win32::UI::WindowsAndMessaging::WM_TIMER {
            if let Some(ref b) = bridge {
                let exclusive = b.check_exclusive_mode();
                if exclusive && !exclusive_mode_active {
                    exclusive_mode_active = true;
                    notification::show_exclusive_mode_active();
                } else if !exclusive && exclusive_mode_active {
                    exclusive_mode_active = false;
                    notification::show_exclusive_mode_ended();
                }
            }

            // Notify once when a new version is detected
            if !update_notified {
                if let Some(tag) = update_state.lock().unwrap().clone() {
                    update_notified = true;
                    let url =
                        format!("https://github.com/jeffreytse/winsoftvol/releases/tag/{tag}");
                    notification::show_update_available(&tag, &url);
                }
            }

            // Hot-reload config when file changes externally
            let cfg_path = config::Config::path();
            if let Ok(meta) = std::fs::metadata(&cfg_path) {
                if let Ok(mtime) = meta.modified() {
                    let now = std::time::SystemTime::now();
                    let age = now.duration_since(mtime).unwrap_or_default();
                    // Only reload if file changed since last load and has settled (>500ms)
                    if Some(mtime) != last_config_mtime
                        && age >= std::time::Duration::from_millis(500)
                    {
                        last_config_mtime = Some(mtime);
                        match config::Config::try_load() {
                            Ok(new_cfg) => {
                                softvol_flag
                                    .store(new_cfg.default.force_sw_volume, Ordering::Relaxed);
                                cap_flag.store(new_cfg.default.cap_percent, Ordering::Relaxed);
                                tray_state.set_softvol(new_cfg.default.force_sw_volume);
                                tray_state.set_volcap(new_cfg.default.cap_percent);
                                let old_autostart = cfg_state.read().unwrap().general.autostart;
                                if new_cfg.general.autostart != old_autostart {
                                    let _ = autostart::set(new_cfg.general.autostart);
                                    tray_state.set_autostart(new_cfg.general.autostart);
                                }
                                *cfg_state.write().unwrap() = new_cfg;
                            }
                            Err(e) => {
                                notification::show_config_error(&e.to_string());
                            }
                        }
                    }
                }
            }
        }

        if watcher.check() {
            drop(bridge.take());
            bridge = audio::AudioBridge::new(softvol_flag.clone(), cap_flag.clone()).ok();
            if bridge.is_some() {
                notification::show_device_reconnected();
            }
        }

        while let Ok(event) = tray_icon::TrayIconEvent::receiver().try_recv() {
            match event {
                tray_icon::TrayIconEvent::Click {
                    button: tray_icon::MouseButton::Left,
                    button_state: tray_icon::MouseButtonState::Up,
                    ..
                } => {
                    if let Some(ref b) = bridge {
                        let _ = b.toggle_mute();
                    }
                }
                tray_icon::TrayIconEvent::Enter { .. } => {
                    CURSOR_OVER_TRAY.store(true, Ordering::Relaxed);
                }
                tray_icon::TrayIconEvent::Leave { .. } => {
                    CURSOR_OVER_TRAY.store(false, Ordering::Relaxed);
                }
                _ => {}
            }
        }

        while let Ok(event) = muda::MenuEvent::receiver().try_recv() {
            if event.id() == &tray_state.about_id {
                let latest = update_state.lock().unwrap().clone();
                about::show_about(latest.as_deref());
            } else if event.id() == &tray_state.quit_id {
                unsafe {
                    let _ = KillTimer(HWND(0), 1);
                }
                drop(bridge.take());
                unsafe {
                    let _ = UnhookWindowsHookEx(hook);
                }
                return Ok(());
            } else if event.id() == &tray_state.autostart_id {
                let new_state = !cfg_state.read().unwrap().general.autostart;
                if let Err(e) = autostart::set(new_state) {
                    eprintln!("autostart error: {e}");
                }
                {
                    let mut cfg = cfg_state.write().unwrap();
                    cfg.general.autostart = new_state;
                    let _ = cfg.save();
                }
            } else if event.id() == &tray_state.softvol_id {
                let new_state = !softvol_flag.load(Ordering::Relaxed);
                softvol_flag.store(new_state, Ordering::Relaxed);
                {
                    let mut cfg = cfg_state.write().unwrap();
                    cfg.default.force_sw_volume = new_state;
                    let _ = cfg.save();
                }
            } else {
                for (id, pct) in &tray_state.volcap_ids {
                    if event.id() == id {
                        cap_flag.store(*pct, Ordering::Relaxed);
                        {
                            let mut cfg = cfg_state.write().unwrap();
                            cfg.default.cap_percent = *pct;
                            let _ = cfg.save();
                        }
                        tray_state.set_volcap(*pct);
                        break;
                    }
                }
            }
        }
    }

    unsafe {
        let _ = KillTimer(HWND(0), 1);
        let _ = UnhookWindowsHookEx(hook);
    }
    drop(bridge.take());
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("winsoftvol only runs on Windows");
    std::process::exit(1);
}

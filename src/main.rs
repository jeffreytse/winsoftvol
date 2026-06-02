#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
mod about;
#[cfg(windows)]
mod audio;
#[cfg(windows)]
mod autostart;
#[cfg(windows)]
mod softvol;
#[cfg(windows)]
mod volcap;
#[cfg(windows)]
mod notification;
mod tray;

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
        CallNextHookEx, HHOOK, WM_MOUSEWHEEL, MSLLHOOKSTRUCT,
    };
    if code >= 0 && wparam.0 as u32 == WM_MOUSEWHEEL {
        if CURSOR_OVER_TRAY.load(Ordering::Relaxed) {
            let data = &*(lparam.0 as *const MSLLHOOKSTRUCT);
            let delta = (data.mouseData >> 16) as i16;
            if delta > 0 {
                PENDING_SCROLL.fetch_add(1, Ordering::Relaxed);
            } else {
                PENDING_SCROLL.fetch_sub(1, Ordering::Relaxed);
            }
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
    use std::sync::{
        atomic::AtomicU32,
        Arc,
    };
    use windows::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, KillTimer, SetTimer, TranslateMessage,
            SetWindowsHookExW, UnhookWindowsHookEx, WH_MOUSE_LL, MSG,
        },
    };

    notification::register_aumid();

    let softvol_flag = Arc::new(AtomicBool::new(softvol::is_enabled()));
    let cap_flag = Arc::new(AtomicU32::new(volcap::get()));

    let watcher = audio::DeviceWatcher::new()?;
    let mut bridge: Option<audio::AudioBridge> =
        audio::AudioBridge::new(softvol_flag.clone(), cap_flag.clone()).ok();
    let tray_state =
        tray::build_tray(autostart::is_enabled(), softvol::is_enabled(), volcap::get())?;

    let hook = unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), None, 0)? };

    unsafe { SetTimer(HWND(0), 1, 1000, None) };

    let mut last_display: Option<(u32, bool)> = None;
    let mut exclusive_mode_active = false;
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

        // Check for exclusive mode once per timer tick (1 s)
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
                about::show_about();
            } else if event.id() == &tray_state.quit_id {
                unsafe {
                    let _ = KillTimer(HWND(0), 1);
                }
                drop(bridge.take());
                unsafe { let _ = UnhookWindowsHookEx(hook); }
                return Ok(());
            } else if event.id() == &tray_state.autostart_id {
                let new_state = !autostart::is_enabled();
                if let Err(e) = autostart::set(new_state) {
                    eprintln!("autostart error: {e}");
                }
            } else if event.id() == &tray_state.softvol_id {
                let new_state = !softvol::is_enabled();
                softvol_flag.store(new_state, Ordering::Relaxed);
                if let Err(e) = softvol::set(new_state) {
                    eprintln!("softvol error: {e}");
                }
            } else {
                for (id, pct) in &tray_state.volcap_ids {
                    if event.id() == id {
                        cap_flag.store(*pct, Ordering::Relaxed);
                        if let Err(e) = volcap::set(*pct) {
                            eprintln!("volcap error: {e}");
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

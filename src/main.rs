#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
mod about;
#[cfg(windows)]
mod audio;
#[cfg(windows)]
mod autostart;
#[cfg(windows)]
mod softvol;
mod tray;

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
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use windows::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, KillTimer, SetTimer, TranslateMessage, MSG,
        },
    };

    let softvol_flag = Arc::new(AtomicBool::new(softvol::is_enabled()));

    let watcher = audio::DeviceWatcher::new()?;
    let mut bridge: Option<audio::AudioBridge> = audio::AudioBridge::new(softvol_flag.clone()).ok();
    let tray_state = tray::build_tray(autostart::is_enabled(), softvol::is_enabled())?;

    unsafe { SetTimer(HWND(0), 1, 1000, None) };

    let mut msg = MSG::default();
    loop {
        unsafe {
            if !GetMessageW(&mut msg, None, 0, 0).as_bool() {
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if watcher.check() {
            drop(bridge.take());
            bridge = audio::AudioBridge::new(softvol_flag.clone()).ok();
        }

        while let Ok(event) = muda::MenuEvent::receiver().try_recv() {
            if event.id() == &tray_state.about_id {
                about::show_about();
            } else if event.id() == &tray_state.quit_id {
                unsafe {
                    let _ = KillTimer(HWND(0), 1);
                }
                drop(bridge.take());
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
            }
        }
    }

    unsafe {
        let _ = KillTimer(HWND(0), 1);
    }
    drop(bridge.take());
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("winsoftvol only runs on Windows");
    std::process::exit(1);
}

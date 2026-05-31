#[cfg(windows)]
pub fn show_about() {
    use windows::{
        core::PCWSTR,
        Win32::{
            Foundation::HWND,
            UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_OK},
        },
    };

    let text = format!(
        "Sound Control  v{} ({})\n\n{}\n\nAuthor:  {}\nBuilt:   {}",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_HASH"),
        env!("CARGO_PKG_DESCRIPTION"),
        env!("CARGO_PKG_AUTHORS"),
        env!("BUILD_TIME"),
    );

    let text_w: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
    let title_w: Vec<u16> = "About Sound Control\0".encode_utf16().collect();

    unsafe {
        MessageBoxW(
            HWND(0),
            PCWSTR(text_w.as_ptr()),
            PCWSTR(title_w.as_ptr()),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}

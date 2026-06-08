fn null_terminated_utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

#[cfg(windows)]
unsafe extern "system" fn hyperlink_callback(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    _wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
    data: isize,
) -> windows::core::HRESULT {
    use windows::{
        core::PCWSTR,
        Win32::{
            Foundation::S_OK,
            UI::{
                Controls::{TDN_CREATED, TDN_HYPERLINK_CLICKED},
                Shell::ShellExecuteW,
                WindowsAndMessaging::{
                    SendMessageW, ICON_BIG, ICON_SMALL, SW_SHOWNORMAL, WM_SETICON,
                },
            },
        },
    };
    if msg == TDN_HYPERLINK_CLICKED.0 as u32 {
        let url = PCWSTR(lparam.0 as *const u16);
        ShellExecuteW(
            None,
            windows::core::w!("open"),
            url,
            PCWSTR(std::ptr::null()),
            PCWSTR(std::ptr::null()),
            SW_SHOWNORMAL,
        );
    } else if msg == TDN_CREATED.0 as u32 {
        let hicon = data as *mut std::ffi::c_void;
        SendMessageW(
            hwnd,
            WM_SETICON,
            windows::Win32::Foundation::WPARAM(ICON_SMALL as usize),
            windows::Win32::Foundation::LPARAM(hicon as isize),
        );
        SendMessageW(
            hwnd,
            WM_SETICON,
            windows::Win32::Foundation::WPARAM(ICON_BIG as usize),
            windows::Win32::Foundation::LPARAM(hicon as isize),
        );
    }
    S_OK
}

#[cfg(windows)]
pub fn show_about(latest_version: Option<&str>) {
    use std::mem::size_of;
    use windows::{
        core::PCWSTR,
        Win32::{
            System::LibraryLoader::GetModuleHandleW,
            UI::{
                Controls::{
                    TaskDialogIndirect, TASKDIALOGCONFIG, TASKDIALOG_FLAGS, TDCBF_OK_BUTTON,
                    TDF_ALLOW_DIALOG_CANCELLATION, TDF_ENABLE_HYPERLINKS, TDF_USE_HICON_MAIN,
                },
                WindowsAndMessaging::LoadIconW,
            },
        },
    };

    const HOMEPAGE: &str = "https://github.com/jeffreytse/winsoftvol";
    const SPONSOR: &str = "https://github.com/sponsors/jeffreytse";

    let update_line = latest_version
        .map(|tag| {
            let url = format!("https://github.com/jeffreytse/winsoftvol/releases/tag/{tag}");
            format!("\n\n<a href=\"{url}\">\u{1F195} New version {tag} available \u{2014} click to download</a>")
        })
        .unwrap_or_default();

    let content = format!(
        "v{} ({})\n{}\n\nAuthor:  {}\nBuilt:   {}\n\n<a href=\"{HOMEPAGE}\">Project Homepage</a>\n\nIf you find WinSoftVol useful, please consider supporting its development.\n<a href=\"{SPONSOR}\">Sponsor on GitHub \u{2665}</a>{update_line}",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_HASH"),
        env!("CARGO_PKG_DESCRIPTION"),
        env!("CARGO_PKG_AUTHORS"),
        env!("BUILD_TIME"),
    );

    let title_w = null_terminated_utf16("About WinSoftVol");
    let content_w = null_terminated_utf16(&content);

    let hicon = unsafe {
        GetModuleHandleW(None)
            .ok()
            .and_then(|hmod| LoadIconW(hmod, PCWSTR(std::ptr::dangling::<u16>())).ok())
    };

    let mut config: TASKDIALOGCONFIG = unsafe { std::mem::zeroed() };
    config.cbSize = size_of::<TASKDIALOGCONFIG>() as u32;
    config.dwFlags = TASKDIALOG_FLAGS(TDF_ENABLE_HYPERLINKS.0 | TDF_ALLOW_DIALOG_CANCELLATION.0);
    config.dwCommonButtons = TDCBF_OK_BUTTON;
    config.pszWindowTitle = PCWSTR(title_w.as_ptr());
    config.pszContent = PCWSTR(content_w.as_ptr());
    config.pfCallback = Some(hyperlink_callback);

    if let Some(icon) = hicon {
        config.dwFlags.0 |= TDF_USE_HICON_MAIN.0;
        config.Anonymous1.hMainIcon = icon;
        config.lpCallbackData = icon.0;
    }

    unsafe {
        let _ = TaskDialogIndirect(&config, None, None, None);
    }
}

#[cfg(test)]
mod tests {
    use super::null_terminated_utf16;

    #[test]
    fn null_terminated_ends_with_zero() {
        let v = null_terminated_utf16("hello");
        assert_eq!(*v.last().unwrap(), 0u16);
    }

    #[test]
    fn null_terminated_length_is_chars_plus_one() {
        let s = "WinSoftVol";
        let v = null_terminated_utf16(s);
        assert_eq!(v.len(), s.encode_utf16().count() + 1);
    }

    #[test]
    fn null_terminated_empty_string() {
        let v = null_terminated_utf16("");
        assert_eq!(v, vec![0u16]);
    }

    #[test]
    fn null_terminated_ascii_encodes_correctly() {
        let v = null_terminated_utf16("A");
        assert_eq!(v, vec![65u16, 0u16]);
    }

    #[test]
    fn null_terminated_no_interior_nulls_for_ascii() {
        let v = null_terminated_utf16("hello");
        assert_eq!(v[..v.len() - 1].iter().filter(|&&c| c == 0).count(), 0);
    }
}

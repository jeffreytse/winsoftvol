fn null_terminated_utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

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
        "WinSoftVol  v{} ({})\n\n{}\n\nAuthor:  {}\nBuilt:   {}",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_HASH"),
        env!("CARGO_PKG_DESCRIPTION"),
        env!("CARGO_PKG_AUTHORS"),
        env!("BUILD_TIME"),
    );

    let text_w = null_terminated_utf16(&text);
    let title_w = null_terminated_utf16("About WinSoftVol");

    unsafe {
        MessageBoxW(
            HWND(0),
            PCWSTR(text_w.as_ptr()),
            PCWSTR(title_w.as_ptr()),
            MB_OK | MB_ICONINFORMATION,
        );
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
        assert_eq!(v.len(), s.len() + 1);
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
        // only the trailing null
        assert_eq!(v[..v.len() - 1].iter().filter(|&&c| c == 0).count(), 0);
    }
}

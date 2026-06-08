use std::sync::{Arc, Mutex};

use windows::{
    core::{w, HSTRING, PCWSTR},
    Win32::Networking::WinHttp::{
        WinHttpCloseHandle, WinHttpConnect, WinHttpOpen, WinHttpOpenRequest, WinHttpReadData,
        WinHttpReceiveResponse, WinHttpSendRequest, WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
        WINHTTP_FLAG_SECURE,
    },
};

const REPO: &str = "jeffreytse/winsoftvol";

pub fn check_latest_release() -> Option<String> {
    let json = winhttp_get("api.github.com", &format!("/repos/{REPO}/releases/latest"))?;
    let tag = extract_tag_name(&json)?.to_string();
    if is_newer(tag.trim_start_matches('v'), env!("CARGO_PKG_VERSION")) {
        Some(tag)
    } else {
        None
    }
}

fn winhttp_get(host: &str, path: &str) -> Option<String> {
    unsafe {
        let agent = HSTRING::from(format!("winsoftvol/{}", env!("CARGO_PKG_VERSION")));
        let session = WinHttpOpen(
            &agent,
            WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
            PCWSTR::null(),
            PCWSTR::null(),
            0,
        );
        if session.is_null() {
            return None;
        }

        let host_w = HSTRING::from(host);
        let conn = WinHttpConnect(session, &host_w, 443, 0);
        if conn.is_null() {
            let _ = WinHttpCloseHandle(session);
            return None;
        }

        let path_w = HSTRING::from(path);
        let request = WinHttpOpenRequest(
            conn,
            w!("GET"),
            &path_w,
            PCWSTR::null(),
            PCWSTR::null(),
            std::ptr::null(),
            WINHTTP_FLAG_SECURE,
        );
        if request.is_null() {
            let _ = WinHttpCloseHandle(conn);
            let _ = WinHttpCloseHandle(session);
            return None;
        }

        let ua: Vec<u16> = format!("User-Agent: winsoftvol/{}\r\n", env!("CARGO_PKG_VERSION"))
            .encode_utf16()
            .collect();
        let body = WinHttpSendRequest(request, Some(ua.as_slice()), None, 0, 0, 0)
            .ok()
            .and_then(|_| WinHttpReceiveResponse(request, std::ptr::null_mut()).ok())
            .and_then(|_| {
                let mut body = Vec::new();
                let mut buf = [0u8; 8192];
                loop {
                    let mut read = 0u32;
                    match WinHttpReadData(
                        request,
                        buf.as_mut_ptr().cast(),
                        buf.len() as u32,
                        &mut read,
                    ) {
                        Ok(_) if read == 0 => break,
                        Ok(_) => body.extend_from_slice(&buf[..read as usize]),
                        Err(_) => break,
                    }
                }
                String::from_utf8(body).ok()
            });

        let _ = WinHttpCloseHandle(request);
        let _ = WinHttpCloseHandle(conn);
        let _ = WinHttpCloseHandle(session);

        body
    }
}

fn extract_tag_name(json: &str) -> Option<&str> {
    let key = "\"tag_name\":\"";
    let start = json.find(key)? + key.len();
    let end = start + json[start..].find('"')?;
    Some(&json[start..end])
}

fn is_newer(remote: &str, current: &str) -> bool {
    parse_ver(remote) > parse_ver(current)
}

fn parse_ver(v: &str) -> (u32, u32, u32) {
    let mut parts = v.splitn(3, '.').map(|p| p.parse().unwrap_or(0));
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

pub fn spawn_update_checker(state: Arc<Mutex<Option<String>>>) {
    std::thread::spawn(move || loop {
        if let Some(tag) = check_latest_release() {
            *state.lock().unwrap() = Some(tag);
            return;
        }
        std::thread::sleep(std::time::Duration::from_secs(24 * 3600));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ver_standard() {
        assert_eq!(parse_ver("1.2.3"), (1, 2, 3));
    }

    #[test]
    fn parse_ver_leading_zeros() {
        assert_eq!(parse_ver("0.2.0"), (0, 2, 0));
    }

    #[test]
    fn is_newer_true_when_remote_higher() {
        assert!(is_newer("0.3.0", "0.2.0"));
    }

    #[test]
    fn is_newer_false_when_same() {
        assert!(!is_newer("0.2.0", "0.2.0"));
    }

    #[test]
    fn is_newer_false_when_older() {
        assert!(!is_newer("0.1.0", "0.2.0"));
    }

    #[test]
    fn is_newer_patch_version() {
        assert!(is_newer("0.2.1", "0.2.0"));
    }

    #[test]
    fn extract_tag_name_standard() {
        let json = r#"{"tag_name":"v0.3.0","name":"Release v0.3.0"}"#;
        assert_eq!(extract_tag_name(json), Some("v0.3.0"));
    }

    #[test]
    fn extract_tag_name_missing() {
        assert_eq!(extract_tag_name("{}"), None);
    }
}

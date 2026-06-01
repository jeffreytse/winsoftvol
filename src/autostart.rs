use winreg::{enums::*, RegKey};

const APP_NAME: &str = "WinSoftVol";
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

pub fn is_enabled() -> bool {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(RUN_KEY)
        .and_then(|k| k.get_value::<String, _>(APP_NAME))
        .is_ok()
}

pub fn set(enable: bool) -> anyhow::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(RUN_KEY)?;
    if enable {
        let exe = std::env::current_exe()?;
        key.set_value(APP_NAME, &exe.to_string_lossy().into_owned())?;
    } else {
        let _ = key.delete_value(APP_NAME);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_name_matches_registry_key() {
        assert_eq!(APP_NAME, "WinSoftVol");
    }

    #[test]
    fn run_key_path_is_correct() {
        assert!(RUN_KEY.contains(r"CurrentVersion\Run"));
    }

    #[cfg(windows)]
    #[test]
    fn round_trip() {
        let original = is_enabled();
        set(true).expect("enable autostart");
        assert!(is_enabled(), "should be enabled after set(true)");
        set(false).expect("disable autostart");
        assert!(!is_enabled(), "should be disabled after set(false)");
        set(original).unwrap();
    }
}

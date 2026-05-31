use winreg::{enums::*, RegKey};

const APP_NAME: &str = "SoundControl";
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

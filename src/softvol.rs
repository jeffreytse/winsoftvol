use winreg::{enums::*, RegKey};

const APP_KEY: &str = r"Software\WinSoftVol";
const VALUE: &str = "ForceSwVolume";

pub fn is_enabled() -> bool {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(APP_KEY)
        .and_then(|k| k.get_value::<u32, _>(VALUE))
        .map(|v: u32| v != 0)
        .unwrap_or(false)
}

pub fn set(enable: bool) -> anyhow::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(APP_KEY)?;
    key.set_value(VALUE, &(enable as u32))?;
    Ok(())
}

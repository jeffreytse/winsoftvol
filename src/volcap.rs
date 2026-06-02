use winreg::{enums::*, RegKey};

const APP_KEY: &str = r"Software\WinSoftVol";
const VALUE: &str = "VolumeCapPercent";
pub const DEFAULT: u32 = 100;

pub fn get() -> u32 {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(APP_KEY)
        .and_then(|k| k.get_value::<u32, _>(VALUE))
        .unwrap_or(DEFAULT)
        .clamp(10, 100)
}

pub fn set(percent: u32) -> anyhow::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(APP_KEY)?;
    key.set_value(VALUE, &percent.clamp(10, 100))?;
    Ok(())
}

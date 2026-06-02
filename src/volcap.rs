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

#[cfg(test)]
mod tests {
    use super::{APP_KEY, DEFAULT, VALUE};

    #[test]
    fn default_is_100() {
        assert_eq!(DEFAULT, 100);
    }

    #[test]
    fn clamp_below_minimum() {
        assert_eq!(0u32.clamp(10, 100), 10);
    }

    #[test]
    fn clamp_above_maximum() {
        assert_eq!(150u32.clamp(10, 100), 100);
    }

    #[test]
    fn in_range_passes_through() {
        assert_eq!(80u32.clamp(10, 100), 80);
    }

    #[test]
    fn boundary_values_accepted() {
        assert_eq!(10u32.clamp(10, 100), 10);
        assert_eq!(100u32.clamp(10, 100), 100);
    }

    #[test]
    fn app_key_path_is_correct() {
        assert_eq!(APP_KEY, r"Software\WinSoftVol");
    }

    #[test]
    fn value_name_is_correct() {
        assert_eq!(VALUE, "VolumeCapPercent");
    }
}

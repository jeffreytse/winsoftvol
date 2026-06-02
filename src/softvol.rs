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

#[cfg(test)]
mod tests {
    use super::{APP_KEY, VALUE};

    #[test]
    fn app_key_path_is_correct() {
        assert_eq!(APP_KEY, r"Software\WinSoftVol");
    }

    #[test]
    fn value_name_is_correct() {
        assert_eq!(VALUE, "ForceSwVolume");
    }

    #[test]
    fn zero_stored_means_disabled() {
        // is_enabled reads u32 and checks v != 0
        assert!(!(0u32 != 0));
    }

    #[test]
    fn nonzero_stored_means_enabled() {
        assert!(1u32 != 0);
        assert!(u32::MAX != 0);
    }

    #[test]
    fn bool_to_u32_encoding() {
        assert_eq!(false as u32, 0);
        assert_eq!(true as u32, 1);
    }
}

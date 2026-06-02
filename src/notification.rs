use windows::{
    core::{Result, HSTRING},
    Data::Xml::Dom::XmlDocument,
    UI::Notifications::{ToastNotification, ToastNotificationManager},
};

const AUMID: &str = "WinSoftVol";

/// Register AppUserModelId in HKCU so Windows associates toasts with this app.
/// Must be called once at startup before showing any toast.
pub fn register_aumid() {
    use winreg::{enums::*, RegKey};
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok((key, _)) = hkcu.create_subkey(r"SOFTWARE\Classes\AppUserModelId\WinSoftVol") {
        let _ = key.set_value("DisplayName", &"WinSoftVol");
    }
}

pub fn show_device_reconnected() {
    let _ = toast(
        "WinSoftVol",
        "USB audio device reconnected — volume control restored.",
    );
}

pub fn show_exclusive_mode_active() {
    let _ = toast(
        "WinSoftVol — exclusive mode detected",
        "An app bypassed the audio mixer. Volume control won't apply to it until it releases the device.",
    );
}

pub fn show_exclusive_mode_ended() {
    let _ = toast(
        "WinSoftVol",
        "Exclusive audio mode ended — volume control restored.",
    );
}

fn toast(title: &str, body: &str) -> Result<()> {
    let xml = XmlDocument::new()?;
    xml.LoadXml(&HSTRING::from(format!(
        "<toast duration=\"short\"><visual><binding template=\"ToastGeneric\"><text>{title}</text><text>{body}</text></binding></visual></toast>"
    )))?;
    let notif = ToastNotification::CreateToastNotification(&xml)?;
    ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(AUMID))?.Show(&notif)?;
    Ok(())
}

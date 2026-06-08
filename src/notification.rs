use windows::{
    core::{Result, HSTRING},
    Data::Xml::Dom::XmlDocument,
    UI::Notifications::{ToastNotification, ToastNotificationManager},
};

const AUMID: &str = "WinSoftVol";

const TITLE_APP: &str = "WinSoftVol";
const TITLE_EXCLUSIVE: &str = "WinSoftVol — exclusive mode detected";
const MSG_RECONNECTED: &str = "USB audio device reconnected — volume control restored.";
const MSG_EXCLUSIVE_START: &str = "An app bypassed the audio mixer. Volume control won't apply to it until it releases the device.";
const MSG_EXCLUSIVE_END: &str = "Exclusive audio mode ended — volume control restored.";

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
    let _ = toast(TITLE_APP, MSG_RECONNECTED);
}

pub fn show_config_error(msg: &str) {
    let truncated: String = msg.chars().take(200).collect();
    let body =
        format!("config.toml: {truncated}\nPrevious settings kept. Fix the file to apply changes.");
    let _ = toast("WinSoftVol — Config Error", &body);
}

pub fn show_exclusive_mode_active() {
    let _ = toast(TITLE_EXCLUSIVE, MSG_EXCLUSIVE_START);
}

pub fn show_exclusive_mode_ended() {
    let _ = toast(TITLE_APP, MSG_EXCLUSIVE_END);
}

fn build_toast_xml(title: &str, body: &str) -> String {
    format!(
        "<toast duration=\"short\"><visual><binding template=\"ToastGeneric\"><text>{title}</text><text>{body}</text></binding></visual></toast>"
    )
}

fn toast(title: &str, body: &str) -> Result<()> {
    toast_xml(&build_toast_xml(title, body))
}

fn toast_xml(xml_str: &str) -> Result<()> {
    let xml = XmlDocument::new()?;
    xml.LoadXml(&HSTRING::from(xml_str))?;
    let notif = ToastNotification::CreateToastNotification(&xml)?;
    ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(AUMID))?.Show(&notif)?;
    Ok(())
}

pub fn show_update_available(tag: &str, url: &str) {
    let xml = format!(
        "<toast launch=\"{url}\" activationType=\"protocol\" duration=\"short\"><visual><binding template=\"ToastGeneric\"><text>WinSoftVol Update Available</text><text>{tag} is ready \u{2014} click to open release page</text></binding></visual></toast>"
    );
    let _ = toast_xml(&xml);
}

#[cfg(test)]
mod tests {
    use super::{
        build_toast_xml, AUMID, MSG_EXCLUSIVE_END, MSG_EXCLUSIVE_START, MSG_RECONNECTED,
        TITLE_EXCLUSIVE,
    };

    #[test]
    fn xml_contains_duration_short() {
        let xml = build_toast_xml("T", "B");
        assert!(xml.contains("duration=\"short\""));
    }

    #[test]
    fn xml_contains_toast_generic_template() {
        let xml = build_toast_xml("T", "B");
        assert!(xml.contains("template=\"ToastGeneric\""));
    }

    #[test]
    fn xml_title_appears_before_body() {
        let xml = build_toast_xml("TITLE", "BODY");
        let title_pos = xml.find("TITLE").unwrap();
        let body_pos = xml.find("BODY").unwrap();
        assert!(title_pos < body_pos);
    }

    #[test]
    fn xml_wraps_title_in_text_tag() {
        let xml = build_toast_xml("Hello", "World");
        assert!(xml.contains("<text>Hello</text>"));
    }

    #[test]
    fn xml_wraps_body_in_text_tag() {
        let xml = build_toast_xml("Hello", "World");
        assert!(xml.contains("<text>World</text>"));
    }

    #[test]
    fn xml_has_correct_nesting() {
        let xml = build_toast_xml("T", "B");
        // toast > visual > binding > text
        assert!(xml.starts_with("<toast"));
        assert!(xml.contains("<visual>"));
        assert!(xml.contains("<binding "));
        assert!(xml.ends_with("</toast>"));
    }

    #[test]
    fn aumid_is_winsoftvol() {
        assert_eq!(AUMID, "WinSoftVol");
    }

    #[test]
    fn msg_reconnected_mentions_reconnected_and_restored() {
        assert!(MSG_RECONNECTED.contains("reconnected"));
        assert!(MSG_RECONNECTED.contains("restored"));
    }

    #[test]
    fn msg_exclusive_start_mentions_bypassed() {
        assert!(MSG_EXCLUSIVE_START.contains("bypassed"));
    }

    #[test]
    fn msg_exclusive_end_mentions_ended_and_restored() {
        assert!(MSG_EXCLUSIVE_END.contains("ended"));
        assert!(MSG_EXCLUSIVE_END.contains("restored"));
    }

    #[test]
    fn title_exclusive_mentions_exclusive() {
        assert!(TITLE_EXCLUSIVE.contains("exclusive"));
    }
}

use std::collections::HashMap;

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub default: DeviceConfig,
    #[serde(default)]
    pub device: HashMap<String, DeviceConfig>,
}

fn default_cap_presets() -> Vec<u32> {
    vec![100, 80, 60, 40]
}

fn default_scroll_step() -> u32 {
    2
}

fn default_night_cap() -> u32 {
    40
}

fn default_true() -> bool {
    true
}

fn parse_hm(s: &str) -> Option<u32> {
    let (h, m) = s.split_once(':')?;
    let h: u32 = h.trim().parse().ok()?;
    let m: u32 = m.trim().parse().ok()?;
    if h > 23 || m > 59 {
        return None;
    }
    Some(h * 60 + m)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default)]
    pub autostart: bool,
    #[serde(default = "default_cap_presets")]
    pub cap_presets: Vec<u32>,
    #[serde(default = "default_scroll_step")]
    pub scroll_step_percent: u32,
    #[serde(default)]
    pub night_start: Option<String>,
    #[serde(default)]
    pub night_end: Option<String>,
    #[serde(default = "default_night_cap")]
    pub night_cap: u32,
    #[serde(default = "default_true")]
    pub night_enabled: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            autostart: false,
            cap_presets: default_cap_presets(),
            scroll_step_percent: default_scroll_step(),
            night_start: None,
            night_end: None,
            night_cap: default_night_cap(),
            night_enabled: true,
        }
    }
}

impl GeneralConfig {
    fn sanitize(&mut self) {
        let mut seen = std::collections::HashSet::new();
        self.cap_presets = self
            .cap_presets
            .iter()
            .map(|&p| p.clamp(10, 100))
            .filter(|p| seen.insert(*p))
            .collect();
        self.cap_presets.sort_unstable_by(|a, b| b.cmp(a));
        if self.cap_presets.is_empty() {
            self.cap_presets = default_cap_presets();
        }
        self.scroll_step_percent = self.scroll_step_percent.clamp(1, 20);
        self.night_cap = self.night_cap.clamp(10, 100);
    }

    pub fn night_window_minutes(&self) -> Option<(u32, u32)> {
        if !self.night_enabled {
            return None;
        }
        let start = parse_hm(self.night_start.as_deref()?)?;
        let end = parse_hm(self.night_end.as_deref()?)?;
        Some((start, end))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub force_sw_volume: bool,
    pub cap_percent: u32,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            force_sw_volume: false,
            cap_percent: 100,
        }
    }
}

impl DeviceConfig {
    fn sanitize(&mut self) {
        self.cap_percent = self.cap_percent.clamp(10, 100);
    }
}

impl Config {
    #[allow(dead_code)]
    pub fn resolve_device<'a>(&'a self, device_id: &str) -> &'a DeviceConfig {
        self.device.get(device_id).unwrap_or(&self.default)
    }

    fn sanitize_devices(&mut self) {
        self.general.sanitize();
        self.default.sanitize();
        for dev in self.device.values_mut() {
            dev.sanitize();
        }
    }

    pub fn path() -> std::path::PathBuf {
        if let Ok(p) = std::env::var("WINSOFTVOL_CONFIG") {
            return std::path::PathBuf::from(p);
        }
        #[cfg(windows)]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                return std::path::Path::new(&appdata)
                    .join("WinSoftVol")
                    .join("config.toml");
            }
        }
        // Fallback: exe directory
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("config.toml")))
            .unwrap_or_else(|| std::path::PathBuf::from("config.toml"))
    }

    pub fn load() -> Self {
        let path = Self::path();
        if !path.exists() {
            #[cfg(windows)]
            {
                if let Some(migrated) = Self::migrate_from_registry() {
                    return migrated;
                }
            }
            return Self::default();
        }
        match std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!(e))
            .and_then(|s| toml::from_str::<Self>(&s).map_err(|e| anyhow::anyhow!(e)))
        {
            Ok(mut cfg) => {
                cfg.sanitize_devices();
                cfg
            }
            Err(e) => {
                eprintln!("config load error: {e}");
                Self::default()
            }
        }
    }

    pub fn try_load() -> anyhow::Result<Self> {
        let path = Self::path();
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!(e))
            .map(|mut c: Self| {
                c.sanitize_devices();
                c
            })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let content = toml::to_string_pretty(self)?;
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, &content)?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }

    #[cfg(windows)]
    fn migrate_from_registry() -> Option<Self> {
        use winreg::{enums::*, RegKey};

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let app_key = hkcu.open_subkey(r"Software\WinSoftVol").ok()?;

        let force_sw: u32 = app_key.get_value("ForceSwVolume").unwrap_or(0);
        let cap: u32 = app_key
            .get_value::<u32, _>("VolumeCapPercent")
            .unwrap_or(100)
            .clamp(10, 100);
        let autostart = hkcu
            .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")
            .and_then(|k| k.get_value::<String, _>("WinSoftVol"))
            .is_ok();

        let cfg = Self {
            general: GeneralConfig {
                autostart,
                cap_presets: default_cap_presets(),
                scroll_step_percent: default_scroll_step(),
                night_start: None,
                night_end: None,
                night_cap: default_night_cap(),
                night_enabled: true,
            },
            default: DeviceConfig {
                force_sw_volume: force_sw != 0,
                cap_percent: cap,
            },
            device: HashMap::new(),
        };

        if let Err(e) = cfg.save() {
            eprintln!("config migration save error: {e}");
            return None;
        }

        // Remove old registry values (best-effort)
        let _ = hkcu.delete_subkey_all(r"Software\WinSoftVol");

        Some(cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_scroll_step_is_two() {
        assert_eq!(default_scroll_step(), 2);
        assert_eq!(GeneralConfig::default().scroll_step_percent, 2);
    }

    #[test]
    fn scroll_step_missing_from_toml_uses_default() {
        let cfg: Config = toml::from_str("[general]\nautostart = false\n").unwrap();
        assert_eq!(cfg.general.scroll_step_percent, 2);
    }

    #[test]
    fn scroll_step_sanitize_clamps_to_range() {
        let mut g = GeneralConfig::default();
        g.scroll_step_percent = 50;
        g.sanitize();
        assert_eq!(g.scroll_step_percent, 20);

        g.scroll_step_percent = 0;
        g.sanitize();
        assert_eq!(g.scroll_step_percent, 1);
    }

    #[test]
    fn default_config_has_sane_values() {
        let cfg = Config::default();
        assert!(!cfg.general.autostart);
        assert!(!cfg.default.force_sw_volume);
        assert_eq!(cfg.default.cap_percent, 100);
        assert!(cfg.device.is_empty());
        assert_eq!(cfg.general.cap_presets, vec![100, 80, 60, 40]);
    }

    #[test]
    fn default_cap_presets_are_descending() {
        let presets = default_cap_presets();
        for w in presets.windows(2) {
            assert!(w[0] > w[1]);
        }
    }

    #[test]
    fn default_cap_presets_in_valid_range() {
        for p in default_cap_presets() {
            assert!((10..=100).contains(&p));
        }
    }

    #[test]
    fn cap_presets_missing_from_toml_uses_default() {
        let cfg: Config = toml::from_str("[general]\nautostart = false\n").unwrap();
        assert_eq!(cfg.general.cap_presets, default_cap_presets());
    }

    #[test]
    fn cap_presets_sanitize_clamps_and_deduplicates() {
        let mut g = GeneralConfig::default();
        g.cap_presets = vec![120, 100, 50, 50, 5];
        g.sanitize();
        assert_eq!(g.cap_presets, vec![100, 50, 10]);
    }

    #[test]
    fn cap_presets_sanitize_empty_restores_default() {
        let mut g = GeneralConfig::default();
        g.cap_presets = vec![];
        g.sanitize();
        assert_eq!(g.cap_presets, default_cap_presets());
    }

    #[test]
    fn cap_presets_sanitize_sorts_descending() {
        let mut g = GeneralConfig::default();
        g.cap_presets = vec![40, 100, 60];
        g.sanitize();
        assert_eq!(g.cap_presets, vec![100, 60, 40]);
    }

    #[test]
    fn parse_minimal_toml() {
        let toml = r#"
[general]
autostart = false

[default]
force_sw_volume = false
cap_percent = 100
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        assert!(!cfg.general.autostart);
        assert_eq!(cfg.default.cap_percent, 100);
    }

    #[test]
    fn parse_empty_toml_uses_defaults() {
        let cfg: Config = toml::from_str("").unwrap();
        assert!(!cfg.general.autostart);
        assert_eq!(cfg.default.cap_percent, 100);
    }

    #[test]
    fn parse_device_section() {
        let toml = r#"
[device."USB Audio {GUID}"]
force_sw_volume = true
cap_percent = 80
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        let dev = cfg.device.get("USB Audio {GUID}").unwrap();
        assert!(dev.force_sw_volume);
        assert_eq!(dev.cap_percent, 80);
    }

    #[test]
    fn resolve_device_returns_device_specific_when_present() {
        let toml = r#"
[default]
force_sw_volume = false
cap_percent = 100

[device."my-device"]
force_sw_volume = true
cap_percent = 60
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        let resolved = cfg.resolve_device("my-device");
        assert!(resolved.force_sw_volume);
        assert_eq!(resolved.cap_percent, 60);
    }

    #[test]
    fn resolve_device_falls_back_to_default() {
        let cfg = Config::default();
        let resolved = cfg.resolve_device("unknown-device-id");
        assert_eq!(resolved.cap_percent, 100);
        assert!(!resolved.force_sw_volume);
    }

    #[test]
    fn roundtrip_serialize_deserialize() {
        let mut cfg = Config::default();
        cfg.general.autostart = true;
        cfg.default.cap_percent = 80;
        cfg.device.insert(
            "test-device".to_string(),
            DeviceConfig {
                force_sw_volume: true,
                cap_percent: 60,
            },
        );
        let serialized = toml::to_string_pretty(&cfg).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.general.autostart, cfg.general.autostart);
        assert_eq!(deserialized.default.cap_percent, cfg.default.cap_percent);
        let dev = deserialized.device.get("test-device").unwrap();
        assert!(dev.force_sw_volume);
        assert_eq!(dev.cap_percent, 60);
    }

    #[test]
    fn cap_percent_zero_clamped_to_10_on_load() {
        let toml = "[default]\nforce_sw_volume = false\ncap_percent = 0\n";
        let cfg: Config = toml::from_str(toml).unwrap();
        // raw parse accepts 0; sanitize via load path
        let mut cfg2 = cfg;
        cfg2.sanitize_devices();
        assert_eq!(cfg2.default.cap_percent, 10);
    }

    #[test]
    fn cap_percent_over_100_clamped_on_load() {
        let toml = "[default]\nforce_sw_volume = false\ncap_percent = 150\n";
        let mut cfg: Config = toml::from_str(toml).unwrap();
        cfg.sanitize_devices();
        assert_eq!(cfg.default.cap_percent, 100);
    }

    #[test]
    fn night_cap_default_is_40() {
        assert_eq!(default_night_cap(), 40);
        assert_eq!(GeneralConfig::default().night_cap, 40);
    }

    #[test]
    fn night_fields_missing_from_toml_use_defaults() {
        let cfg: Config = toml::from_str("[general]\nautostart = false\n").unwrap();
        assert!(cfg.general.night_start.is_none());
        assert!(cfg.general.night_end.is_none());
        assert_eq!(cfg.general.night_cap, 40);
        assert!(cfg.general.night_enabled);
    }

    #[test]
    fn parse_hm_valid() {
        assert_eq!(parse_hm("22:00"), Some(22 * 60));
        assert_eq!(parse_hm("07:30"), Some(7 * 60 + 30));
        assert_eq!(parse_hm("00:00"), Some(0));
        assert_eq!(parse_hm("23:59"), Some(23 * 60 + 59));
    }

    #[test]
    fn parse_hm_invalid() {
        assert_eq!(parse_hm("24:00"), None);
        assert_eq!(parse_hm("22:60"), None);
        assert_eq!(parse_hm("not-a-time"), None);
        assert_eq!(parse_hm(""), None);
        assert_eq!(parse_hm("22"), None);
    }

    #[test]
    fn night_window_minutes_none_when_times_missing() {
        let cfg = GeneralConfig::default();
        assert!(cfg.night_window_minutes().is_none());
    }

    #[test]
    fn night_window_minutes_none_when_disabled() {
        let mut cfg = GeneralConfig::default();
        cfg.night_start = Some("22:00".to_string());
        cfg.night_end = Some("07:00".to_string());
        cfg.night_enabled = false;
        assert!(cfg.night_window_minutes().is_none());
    }

    #[test]
    fn night_window_minutes_returns_parsed_values() {
        let mut cfg = GeneralConfig::default();
        cfg.night_start = Some("22:00".to_string());
        cfg.night_end = Some("07:00".to_string());
        assert_eq!(cfg.night_window_minutes(), Some((22 * 60, 7 * 60)));
    }

    #[test]
    fn night_cap_sanitize_clamps() {
        let mut g = GeneralConfig::default();
        g.night_cap = 5;
        g.sanitize();
        assert_eq!(g.night_cap, 10);

        g.night_cap = 150;
        g.sanitize();
        assert_eq!(g.night_cap, 100);
    }

    #[test]
    fn cap_percent_valid_unchanged_on_sanitize() {
        let toml = "[default]\nforce_sw_volume = false\ncap_percent = 80\n";
        let mut cfg: Config = toml::from_str(toml).unwrap();
        cfg.sanitize_devices();
        assert_eq!(cfg.default.cap_percent, 80);
    }

    #[test]
    fn cap_percent_device_override_clamped() {
        let toml = "[device.\"my-dev\"]\nforce_sw_volume = false\ncap_percent = 0\n";
        let mut cfg: Config = toml::from_str(toml).unwrap();
        cfg.sanitize_devices();
        assert_eq!(cfg.device["my-dev"].cap_percent, 10);
    }

    #[test]
    fn path_uses_env_var_when_set() {
        std::env::set_var("WINSOFTVOL_CONFIG", "/tmp/custom/config.toml");
        let p = Config::path();
        std::env::remove_var("WINSOFTVOL_CONFIG");
        assert_eq!(p, std::path::PathBuf::from("/tmp/custom/config.toml"));
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        std::env::set_var("WINSOFTVOL_CONFIG", path.to_str().unwrap());

        let mut cfg = Config::default();
        cfg.general.autostart = true;
        cfg.default.cap_percent = 60;
        cfg.save().unwrap();

        let loaded = Config::load();
        std::env::remove_var("WINSOFTVOL_CONFIG");

        assert_eq!(loaded.general.autostart, true);
        assert_eq!(loaded.default.cap_percent, 60);
    }
}

use muda::{CheckMenuItem, IsMenuItem, Menu, MenuId, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{TrayIcon, TrayIconBuilder};

const ICON: &[u8] = include_bytes!("../assets/icon.png");

pub struct Tray {
    _icon: TrayIcon,
    pub about_id: MenuId,
    pub autostart_id: MenuId,
    pub softvol_id: MenuId,
    pub night_id: MenuId,
    pub volcap_ids: Vec<(MenuId, u32)>,
    autostart_item: CheckMenuItem,
    softvol_item: CheckMenuItem,
    night_item: CheckMenuItem,
    volcap_items: Vec<CheckMenuItem>,
    pub quit_id: MenuId,
}

pub fn build_tray(
    autostart_enabled: bool,
    softvol_enabled: bool,
    night_enabled: bool,
    volcap_percent: u32,
    cap_presets: &[u32],
) -> anyhow::Result<Tray> {
    let about_item = MenuItem::new("About WinSoftVol", true, None);
    let autostart_item =
        CheckMenuItem::new("Start on Windows startup", true, autostart_enabled, None);
    let softvol_item = CheckMenuItem::new("Force software volume", true, softvol_enabled, None);
    let night_item = CheckMenuItem::new("Night mode", true, night_enabled, None);
    let quit_item = MenuItem::new("Quit WinSoftVol", true, None);

    let about_id = about_item.id().clone();
    let autostart_id = autostart_item.id().clone();
    let softvol_id = softvol_item.id().clone();
    let night_id = night_item.id().clone();
    let quit_id = quit_item.id().clone();

    // Max volume submenu — built from config presets
    let mut volcap_items: Vec<CheckMenuItem> = Vec::new();
    let mut volcap_ids: Vec<(MenuId, u32)> = Vec::new();
    for &pct in cap_presets {
        let label = format!("{pct}%");
        let item = CheckMenuItem::new(&label, true, pct == volcap_percent, None);
        volcap_ids.push((item.id().clone(), pct));
        volcap_items.push(item);
    }
    let volcap_dyn: Vec<&dyn IsMenuItem> =
        volcap_items.iter().map(|i| i as &dyn IsMenuItem).collect();
    let volcap_submenu = Submenu::with_items("Max volume", true, &volcap_dyn)?;

    let menu = Menu::new();
    menu.append(&about_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&autostart_item)?;
    menu.append(&softvol_item)?;
    menu.append(&night_item)?;
    menu.append(&volcap_submenu)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&quit_item)?;

    let img = image::load_from_memory(ICON)?.into_rgba8();
    let (w, h) = img.dimensions();
    let icon = tray_icon::Icon::from_rgba(img.into_raw(), w, h)?;

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_tooltip("WinSoftVol — active")
        .with_icon(icon)
        .build()?;

    Ok(Tray {
        _icon: tray,
        about_id,
        autostart_id,
        softvol_id,
        night_id,
        volcap_ids,
        autostart_item,
        softvol_item,
        night_item,
        volcap_items,
        quit_id,
    })
}

fn bar_filled_px(volume: f32, size: u32) -> u32 {
    ((volume.clamp(0.0, 1.0) * size as f32).round() as u32).min(size)
}

const ICON_SIZE: u32 = 32;
const BAR_HEIGHT: u32 = 4;

/// Renders a 32×32 icon with a volume bar overlaid on the bottom 4 rows.
/// Bar is white when active, red when muted.
pub fn render_volume_icon(volume: f32, muted: bool) -> anyhow::Result<tray_icon::Icon> {
    const SIZE: u32 = ICON_SIZE;
    const BAR_H: u32 = BAR_HEIGHT;

    let base = image::load_from_memory(ICON)?.into_rgba8();
    let mut img = image::imageops::resize(&base, SIZE, SIZE, image::imageops::FilterType::Triangle);

    let filled = bar_filled_px(volume, SIZE);
    let bar_color = if muted {
        image::Rgba([210u8, 60, 60, 255])
    } else {
        image::Rgba([255u8, 255u8, 255u8, 230])
    };
    let track_color = image::Rgba([0u8, 0u8, 0u8, 120]);

    for y in (SIZE - BAR_H)..SIZE {
        for x in 0..SIZE {
            img.put_pixel(x, y, if x < filled { bar_color } else { track_color });
        }
    }

    let (w, h) = img.dimensions();
    Ok(tray_icon::Icon::from_rgba(img.into_raw(), w, h)?)
}

impl Tray {
    pub fn update_icon(&self, icon: tray_icon::Icon) -> anyhow::Result<()> {
        self._icon.set_icon(Some(icon))?;
        Ok(())
    }

    pub fn set_tooltip(&self, text: &str) {
        let _ = self._icon.set_tooltip(Some(text));
    }

    #[allow(dead_code)]
    pub fn set_volcap(&self, pct: u32) {
        for (item, &(_, item_pct)) in self.volcap_items.iter().zip(self.volcap_ids.iter()) {
            item.set_checked(item_pct == pct);
        }
    }

    #[allow(dead_code)]
    pub fn set_softvol(&self, enabled: bool) {
        self.softvol_item.set_checked(enabled);
    }

    #[allow(dead_code)]
    pub fn set_night(&self, enabled: bool) {
        self.night_item.set_checked(enabled);
    }

    #[allow(dead_code)]
    pub fn set_autostart(&self, enabled: bool) {
        self.autostart_item.set_checked(enabled);
    }
}

#[cfg(test)]
mod tests {
    use super::bar_filled_px;

    #[test]
    fn bar_zero_volume() {
        assert_eq!(bar_filled_px(0.0, 32), 0);
    }

    #[test]
    fn bar_full_volume() {
        assert_eq!(bar_filled_px(1.0, 32), 32);
    }

    #[test]
    fn bar_half_volume() {
        assert_eq!(bar_filled_px(0.5, 32), 16);
    }

    #[test]
    fn bar_over_one_clamped() {
        assert_eq!(bar_filled_px(2.0, 32), 32);
    }

    #[test]
    fn bar_negative_clamped() {
        assert_eq!(bar_filled_px(-0.5, 32), 0);
    }

    #[test]
    fn render_volume_icon_smoke() {
        assert!(super::render_volume_icon(0.0, false).is_ok());
        assert!(super::render_volume_icon(0.75, true).is_ok());
        assert!(super::render_volume_icon(1.0, false).is_ok());
    }

    #[test]
    fn icon_size_is_32() {
        assert_eq!(super::ICON_SIZE, 32);
    }

    #[test]
    fn bar_height_is_4() {
        assert_eq!(super::BAR_HEIGHT, 4);
    }

    #[test]
    fn bar_height_less_than_icon_size() {
        assert!(super::BAR_HEIGHT < super::ICON_SIZE);
    }

}

use muda::{CheckMenuItem, IsMenuItem, Menu, MenuId, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{TrayIcon, TrayIconBuilder};

const ICON: &[u8] = include_bytes!("../assets/icon.png");

pub const VOLCAP_PRESETS: &[(u32, &str)] = &[
    (100, "100%"),
    (80, "80%"),
    (60, "60%"),
    (40, "40%"),
];

pub struct Tray {
    _icon: TrayIcon,
    pub about_id: MenuId,
    pub autostart_id: MenuId,
    pub softvol_id: MenuId,
    pub volcap_ids: Vec<(MenuId, u32)>,
    pub quit_id: MenuId,
}

pub fn build_tray(
    autostart_enabled: bool,
    softvol_enabled: bool,
    volcap_percent: u32,
) -> anyhow::Result<Tray> {
    let about_item = MenuItem::new("About WinSoftVol", true, None);
    let autostart_item =
        CheckMenuItem::new("Start on Windows startup", true, autostart_enabled, None);
    let softvol_item =
        CheckMenuItem::new("Force software volume", true, softvol_enabled, None);
    let quit_item = MenuItem::new("Quit WinSoftVol", true, None);

    let about_id = about_item.id().clone();
    let autostart_id = autostart_item.id().clone();
    let softvol_id = softvol_item.id().clone();
    let quit_id = quit_item.id().clone();

    // Max volume submenu
    let mut volcap_items: Vec<CheckMenuItem> = Vec::new();
    let mut volcap_ids: Vec<(MenuId, u32)> = Vec::new();
    for &(pct, label) in VOLCAP_PRESETS {
        let item = CheckMenuItem::new(label, true, pct == volcap_percent, None);
        volcap_ids.push((item.id().clone(), pct));
        volcap_items.push(item);
    }
    let volcap_dyn: Vec<&dyn IsMenuItem> = volcap_items.iter().map(|i| i as &dyn IsMenuItem).collect();
    let volcap_submenu = Submenu::with_items("Max volume", true, &volcap_dyn)?;

    let menu = Menu::new();
    menu.append(&about_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&autostart_item)?;
    menu.append(&softvol_item)?;
    menu.append(&volcap_submenu)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&quit_item)?;

    let img = image::load_from_memory(ICON)?.into_rgba8();
    let (w, h) = img.dimensions();
    let icon = tray_icon::Icon::from_rgba(img.into_raw(), w, h)?;

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("WinSoftVol — active")
        .with_icon(icon)
        .build()?;

    Ok(Tray {
        _icon: tray,
        about_id,
        autostart_id,
        softvol_id,
        volcap_ids,
        quit_id,
    })
}

/// Renders a 32×32 icon with a volume bar overlaid on the bottom 4 rows.
/// Bar is white when active, red when muted.
pub fn render_volume_icon(volume: f32, muted: bool) -> anyhow::Result<tray_icon::Icon> {
    const SIZE: u32 = 32;
    const BAR_H: u32 = 4;

    let base = image::load_from_memory(ICON)?.into_rgba8();
    let mut img = image::imageops::resize(&base, SIZE, SIZE, image::imageops::FilterType::Triangle);

    let filled = ((volume.clamp(0.0, 1.0) * SIZE as f32).round() as u32).min(SIZE);
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
}

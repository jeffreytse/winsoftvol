use muda::{CheckMenuItem, Menu, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};

const ICON: &[u8] = include_bytes!("../assets/icon.png");

pub struct Tray {
    _icon: TrayIcon,
    pub about_id: MenuId,
    pub autostart_id: MenuId,
    pub softvol_id: MenuId,
    pub quit_id: MenuId,
}

pub fn build_tray(autostart_enabled: bool, softvol_enabled: bool) -> anyhow::Result<Tray> {
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

    let menu = Menu::new();
    menu.append(&about_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&autostart_item)?;
    menu.append(&softvol_item)?;
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
        quit_id,
    })
}

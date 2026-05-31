use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use windows::{
    core::{Result, PCWSTR},
    Win32::{
        Media::Audio::{EDataFlow, ERole, IMMNotificationClient, IMMNotificationClient_Impl},
        UI::Shell::PropertiesSystem::PROPERTYKEY,
    },
};

#[windows::core::implement(IMMNotificationClient)]
pub struct DeviceChangeNotifier {
    pub changed: Arc<AtomicBool>,
}

impl IMMNotificationClient_Impl for DeviceChangeNotifier {
    fn OnDeviceStateChanged(&self, _: &PCWSTR, _: u32) -> Result<()> {
        self.changed.store(true, Ordering::Relaxed);
        Ok(())
    }
    fn OnDeviceAdded(&self, _: &PCWSTR) -> Result<()> {
        self.changed.store(true, Ordering::Relaxed);
        Ok(())
    }
    fn OnDeviceRemoved(&self, _: &PCWSTR) -> Result<()> {
        self.changed.store(true, Ordering::Relaxed);
        Ok(())
    }
    fn OnDefaultDeviceChanged(&self, _: EDataFlow, _: ERole, _: &PCWSTR) -> Result<()> {
        self.changed.store(true, Ordering::Relaxed);
        Ok(())
    }
    fn OnPropertyValueChanged(&self, _: &PCWSTR, _: &PROPERTYKEY) -> Result<()> {
        Ok(())
    }
}

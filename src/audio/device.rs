use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use windows::{
    core::Result,
    Win32::{
        Media::Audio::{
            eConsole, eRender, IMMDevice, IMMDeviceEnumerator, IMMNotificationClient,
            MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
        },
        System::Com::{CoCreateInstance, CLSCTX_ALL, STGM_READ},
        UI::Shell::PropertiesSystem::PROPERTYKEY,
    },
};

// {A45C254E-DF1C-4EFD-8020-67D146A850E0}, pid 14
fn pkey_device_friendly_name() -> PROPERTYKEY {
    PROPERTYKEY {
        fmtid: windows::core::GUID {
            data1: 0xa45c254e,
            data2: 0xdf1c,
            data3: 0x4efd,
            data4: [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0],
        },
        pid: 14,
    }
}

use super::device_cb::DeviceChangeNotifier;

pub fn get_device_by_name(name: &str) -> Option<IMMDevice> {
    let enumerator: IMMDeviceEnumerator =
        unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()? };
    let collection = unsafe {
        enumerator
            .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
            .ok()?
    };
    let count = unsafe { collection.GetCount().ok()? };
    for i in 0..count {
        let device = unsafe { collection.Item(i).ok()? };
        if device_friendly_name(&device).as_deref() == Some(name) {
            return Some(device);
        }
    }
    None
}

fn device_friendly_name(device: &IMMDevice) -> Option<String> {
    let store = unsafe { device.OpenPropertyStore(STGM_READ).ok()? };
    let pv = unsafe { store.GetValue(&pkey_device_friendly_name()).ok()? };
    unsafe {
        let pwsz = pv.Anonymous.Anonymous.Anonymous.pwszVal;
        if pwsz.is_null() {
            return None;
        }
        pwsz.to_string().ok()
    }
}

pub fn get_default_device() -> Result<IMMDevice> {
    let enumerator: IMMDeviceEnumerator =
        unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
    unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eConsole) }
}

pub struct DeviceWatcher {
    enumerator: IMMDeviceEnumerator,
    _client: IMMNotificationClient,
    changed: Arc<AtomicBool>,
}

impl DeviceWatcher {
    pub fn new() -> Result<Self> {
        let changed = Arc::new(AtomicBool::new(false));
        let enumerator: IMMDeviceEnumerator =
            unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
        let client: IMMNotificationClient = DeviceChangeNotifier {
            changed: changed.clone(),
        }
        .into();
        unsafe { enumerator.RegisterEndpointNotificationCallback(&client)? };
        Ok(Self {
            enumerator,
            _client: client,
            changed,
        })
    }

    pub fn check(&self) -> bool {
        self.changed.swap(false, Ordering::Relaxed)
    }
}

impl Drop for DeviceWatcher {
    fn drop(&mut self) {
        unsafe {
            let _ = self
                .enumerator
                .UnregisterEndpointNotificationCallback(&self._client);
        }
    }
}

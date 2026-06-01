use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use windows::{
    core::Result,
    Win32::{
        Media::Audio::{
            eConsole, eRender, IMMDevice, IMMDeviceEnumerator, IMMNotificationClient,
            MMDeviceEnumerator,
        },
        System::Com::{CoCreateInstance, CLSCTX_ALL},
    },
};

use super::device_cb::DeviceChangeNotifier;

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

mod device;
mod device_cb;
mod endpoint_cb;
mod session_cb;
pub mod session_mgr;

use device::get_default_device;
pub use device::DeviceWatcher;
use endpoint_cb::EndpointVolumeCallback;
use session_cb::SessionNotificationHandler;
use session_mgr::{scale_all_sessions_volume, set_all_sessions_volume};

use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc, Mutex,
};
use windows::{
    core::Result,
    Win32::{
        Media::Audio::{
            Endpoints::{IAudioEndpointVolume, IAudioEndpointVolumeCallback},
            IAudioSessionManager2, IAudioSessionNotification, IMMDevice,
        },
        System::Com::CLSCTX_ALL,
    },
};

pub struct VolumeState {
    pub volume: f32,
    pub muted: bool,
}

pub struct AudioBridge {
    _device: IMMDevice,
    endpoint_volume: IAudioEndpointVolume,
    _endpoint_cb: IAudioEndpointVolumeCallback,
    session_manager: IAudioSessionManager2,
    _session_cb: IAudioSessionNotification,
    state: Arc<Mutex<VolumeState>>,
    softvol: Arc<AtomicBool>,
    cap: Arc<AtomicU32>,
}

impl AudioBridge {
    pub fn new(softvol: Arc<AtomicBool>, cap: Arc<AtomicU32>) -> Result<Self> {
        let state = Arc::new(Mutex::new(VolumeState {
            volume: 1.0,
            muted: false,
        }));

        let device = get_default_device()?;

        // Register session notification BEFORE enumerating to avoid missing sessions
        let session_manager: IAudioSessionManager2 = unsafe { device.Activate(CLSCTX_ALL, None)? };
        let session_cb: IAudioSessionNotification = SessionNotificationHandler {
            state: state.clone(),
        }
        .into();
        unsafe { session_manager.RegisterSessionNotification(&session_cb)? };

        // Register endpoint volume callback — translates OS volume changes to sessions
        let endpoint_volume: IAudioEndpointVolume = unsafe { device.Activate(CLSCTX_ALL, None)? };
        let endpoint_cb: IAudioEndpointVolumeCallback = EndpointVolumeCallback {
            state: state.clone(),
            session_manager: session_manager.clone(),
            endpoint_volume: endpoint_volume.clone(),
            softvol: softvol.clone(),
            cap: cap.clone(),
        }
        .into();
        unsafe { endpoint_volume.RegisterControlChangeNotify(&endpoint_cb)? };

        // Sync initial state from what the endpoint reports
        let init_vol = unsafe { endpoint_volume.GetMasterVolumeLevelScalar()? };
        let init_muted = unsafe { endpoint_volume.GetMute()?.as_bool() };
        {
            let mut s = state.lock().unwrap();
            s.volume = init_vol;
            s.muted = init_muted;
        }
        let init_cap = cap.load(Ordering::Relaxed) as f32 / 100.0;
        set_all_sessions_volume(&session_manager, init_vol, init_muted, init_cap)?;

        Ok(Self {
            _device: device,
            endpoint_volume,
            _endpoint_cb: endpoint_cb,
            session_manager,
            _session_cb: session_cb,
            state,
            softvol,
            cap,
        })
    }

    pub fn adjust_volume(&self, delta: f32) -> Result<()> {
        if self.softvol.load(Ordering::Relaxed) {
            let (old_vol, new_vol, muted) = {
                let mut s = self.state.lock().unwrap();
                let old = s.volume;
                let new = (old + delta).clamp(0.0, 1.0);
                s.volume = new;
                (old, new, s.muted)
            };
            let cap = self.cap.load(Ordering::Relaxed) as f32 / 100.0;
            scale_all_sessions_volume(&self.session_manager, old_vol, new_vol, muted, cap)?;
        } else {
            let current = unsafe { self.endpoint_volume.GetMasterVolumeLevelScalar()? };
            let new_vol = (current + delta).clamp(0.0, 1.0);
            unsafe {
                self.endpoint_volume
                    .SetMasterVolumeLevelScalar(new_vol, std::ptr::null())?;
            }
        }
        Ok(())
    }
}

impl Drop for AudioBridge {
    fn drop(&mut self) {
        unsafe {
            let _ = self
                .endpoint_volume
                .UnregisterControlChangeNotify(&self._endpoint_cb);
            let _ = self
                .session_manager
                .UnregisterSessionNotification(&self._session_cb);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VolumeState;

    #[test]
    fn volume_state_defaults() {
        let s = VolumeState {
            volume: 1.0,
            muted: false,
        };
        assert!((s.volume - 1.0).abs() < f32::EPSILON);
        assert!(!s.muted);
    }
}

use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc, Mutex,
};
use windows::{
    core::{Result, GUID},
    Win32::Media::Audio::{
        Endpoints::{
            IAudioEndpointVolume, IAudioEndpointVolumeCallback,
            IAudioEndpointVolumeCallback_Impl,
        },
        IAudioSessionManager2, AUDIO_VOLUME_NOTIFICATION_DATA,
    },
};

use super::{session_mgr::scale_all_sessions_volume, VolumeState};

// Identifies volume changes we trigger ourselves — prevents infinite callback loop
// when resetting the endpoint to 1.0 in force-software-volume mode.
const OUR_EVENT_CONTEXT: GUID = GUID {
    data1: 0xd4c1e5f2,
    data2: 0x8a3b,
    data3: 0x4d7c,
    data4: [0x9e, 0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x70],
};

#[windows::core::implement(IAudioEndpointVolumeCallback)]
pub struct EndpointVolumeCallback {
    pub state: Arc<Mutex<VolumeState>>,
    pub session_manager: IAudioSessionManager2,
    pub endpoint_volume: IAudioEndpointVolume,
    pub softvol: Arc<AtomicBool>,
    pub cap: Arc<AtomicU32>,
}

impl IAudioEndpointVolumeCallback_Impl for EndpointVolumeCallback {
    fn OnNotify(&self, pnotify: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> Result<()> {
        if pnotify.is_null() {
            return Ok(());
        }
        let data = unsafe { &*pnotify };

        // Skip callbacks we triggered ourselves (endpoint reset to 1.0)
        if data.guidEventContext == OUR_EVENT_CONTEXT {
            return Ok(());
        }

        let new_volume = data.fMasterVolume;
        let muted = data.bMuted.as_bool();

        // Read old volume before updating state — needed for proportional scaling
        let old_volume = {
            let s = self.state.lock().unwrap();
            s.volume
        };

        {
            let mut s = self.state.lock().unwrap();
            s.volume = new_volume;
            s.muted = muted;
        }

        let cap = self.cap.load(Ordering::Relaxed) as f32 / 100.0;
        scale_all_sessions_volume(&self.session_manager, old_volume, new_volume, muted, cap)?;

        // Force software volume: keep endpoint at 1.0 so hardware doesn't also
        // attenuate. All attenuation is handled by the session mixer above.
        if self.softvol.load(Ordering::Relaxed) {
            unsafe {
                let _ = self
                    .endpoint_volume
                    .SetMasterVolumeLevelScalar(1.0, &OUR_EVENT_CONTEXT);
            }
        }

        Ok(())
    }
}

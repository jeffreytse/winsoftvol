use std::sync::{Arc, Mutex};
use windows::{
    core::Result,
    Win32::Media::Audio::{
        Endpoints::{IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl},
        IAudioSessionManager2, AUDIO_VOLUME_NOTIFICATION_DATA,
    },
};

use super::{session_mgr::set_all_sessions_volume, VolumeState};

#[windows::core::implement(IAudioEndpointVolumeCallback)]
pub struct EndpointVolumeCallback {
    pub state: Arc<Mutex<VolumeState>>,
    pub session_manager: IAudioSessionManager2,
}

impl IAudioEndpointVolumeCallback_Impl for EndpointVolumeCallback {
    fn OnNotify(&self, pnotify: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> Result<()> {
        if pnotify.is_null() {
            return Ok(());
        }
        let data = unsafe { &*pnotify };
        let volume = data.fMasterVolume;
        let muted = data.bMuted.as_bool();

        {
            let mut s = self.state.lock().unwrap();
            s.volume = volume;
            s.muted = muted;
        }

        set_all_sessions_volume(&self.session_manager, volume, muted)
    }
}

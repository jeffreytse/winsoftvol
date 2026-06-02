use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
};
use windows::{
    core::{ComInterface, Result},
    Win32::{
        Foundation::BOOL,
        Media::Audio::{
            IAudioSessionControl, IAudioSessionNotification, IAudioSessionNotification_Impl,
            ISimpleAudioVolume,
        },
    },
};

use super::VolumeState;

#[windows::core::implement(IAudioSessionNotification)]
pub struct SessionNotificationHandler {
    pub state: Arc<Mutex<VolumeState>>,
    pub cap: Arc<AtomicU32>,
}

impl IAudioSessionNotification_Impl for SessionNotificationHandler {
    fn OnSessionCreated(&self, newsession: Option<&IAudioSessionControl>) -> Result<()> {
        if let Some(session) = newsession {
            let s = self.state.lock().unwrap();
            let cap = self.cap.load(Ordering::Relaxed) as f32 / 100.0;
            if let Ok(vol) = session.cast::<ISimpleAudioVolume>() {
                unsafe {
                    let _ = vol.SetMasterVolume((s.volume * cap).clamp(0.0, 1.0), std::ptr::null());
                    let _ = vol.SetMute(BOOL::from(s.muted), std::ptr::null());
                }
            }
        }
        Ok(())
    }
}

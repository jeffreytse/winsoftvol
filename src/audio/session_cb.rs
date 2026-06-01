use std::sync::{Arc, Mutex};
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
}

impl IAudioSessionNotification_Impl for SessionNotificationHandler {
    fn OnSessionCreated(&self, newsession: Option<&IAudioSessionControl>) -> Result<()> {
        if let Some(session) = newsession {
            let s = self.state.lock().unwrap();
            if let Ok(vol) = session.cast::<ISimpleAudioVolume>() {
                unsafe {
                    let _ = vol.SetMasterVolume(s.volume, std::ptr::null());
                    let _ = vol.SetMute(BOOL::from(s.muted), std::ptr::null());
                }
            }
        }
        Ok(())
    }
}

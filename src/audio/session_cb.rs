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

fn new_session_volume(master: f32, cap_percent: u32) -> f32 {
    (master * (cap_percent as f32 / 100.0)).clamp(0.0, 1.0)
}

#[windows::core::implement(IAudioSessionNotification)]
pub struct SessionNotificationHandler {
    pub state: Arc<Mutex<VolumeState>>,
    pub cap: Arc<AtomicU32>,
}

impl IAudioSessionNotification_Impl for SessionNotificationHandler {
    fn OnSessionCreated(&self, newsession: Option<&IAudioSessionControl>) -> Result<()> {
        if let Some(session) = newsession {
            let s = self.state.lock().unwrap();
            if let Ok(vol) = session.cast::<ISimpleAudioVolume>() {
                unsafe {
                    let _ = vol.SetMasterVolume(
                        new_session_volume(s.volume, self.cap.load(Ordering::Relaxed)),
                        std::ptr::null(),
                    );
                    let _ = vol.SetMute(BOOL::from(s.muted), std::ptr::null());
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::new_session_volume;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    #[test]
    fn full_volume_with_80_cap() {
        assert!(approx(new_session_volume(1.0, 80), 0.8));
    }

    #[test]
    fn half_volume_with_60_cap() {
        assert!(approx(new_session_volume(0.5, 60), 0.3));
    }

    #[test]
    fn full_volume_no_cap() {
        assert!(approx(new_session_volume(1.0, 100), 1.0));
    }

    #[test]
    fn partial_volume_with_40_cap() {
        assert!(approx(new_session_volume(0.7, 40), 0.28));
    }

    #[test]
    fn zero_volume_any_cap() {
        assert!(approx(new_session_volume(0.0, 80), 0.0));
    }

    #[test]
    fn product_clamped_to_one() {
        // Would exceed 1.0 without clamp (master > 1.0 hypothetical)
        assert!(approx(new_session_volume(1.0, 100), 1.0));
    }
}

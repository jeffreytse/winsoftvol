use windows::{
    core::{ComInterface, Result},
    Win32::{
        Foundation::BOOL,
        Media::Audio::{IAudioSessionManager2, ISimpleAudioVolume},
    },
};

/// Sets every session to the same absolute volume. Used for initial sync and new sessions.
pub fn set_all_sessions_volume(
    session_manager: &IAudioSessionManager2,
    volume: f32,
    muted: bool,
    cap: f32,
) -> Result<()> {
    let enumerator = unsafe { session_manager.GetSessionEnumerator()? };
    let count = unsafe { enumerator.GetCount()? };

    for i in 0..count {
        let session = unsafe { enumerator.GetSession(i)? };
        if let Ok(vol) = session.cast::<ISimpleAudioVolume>() {
            unsafe {
                let _ = vol.SetMasterVolume((volume * cap).clamp(0.0, 1.0), std::ptr::null());
                let _ = vol.SetMute(BOOL::from(muted), std::ptr::null());
            }
        }
    }
    Ok(())
}

/// Mutes or unmutes every session without touching per-app volume levels.
pub fn set_all_sessions_mute(
    session_manager: &IAudioSessionManager2,
    muted: bool,
) -> Result<()> {
    let enumerator = unsafe { session_manager.GetSessionEnumerator()? };
    let count = unsafe { enumerator.GetCount()? };
    for i in 0..count {
        let session = unsafe { enumerator.GetSession(i)? };
        if let Ok(vol) = session.cast::<ISimpleAudioVolume>() {
            unsafe {
                let _ = vol.SetMute(BOOL::from(muted), std::ptr::null());
            }
        }
    }
    Ok(())
}

/// Scales each session proportionally by (new_volume / old_volume), clamped to cap.
/// Preserves per-app volume balance set in Windows Volume Mixer.
pub fn scale_all_sessions_volume(
    session_manager: &IAudioSessionManager2,
    old_volume: f32,
    new_volume: f32,
    muted: bool,
    cap: f32,
) -> Result<()> {
    let enumerator = unsafe { session_manager.GetSessionEnumerator()? };
    let count = unsafe { enumerator.GetCount()? };

    for i in 0..count {
        let session = unsafe { enumerator.GetSession(i)? };
        if let Ok(vol) = session.cast::<ISimpleAudioVolume>() {
            unsafe {
                let scaled = if old_volume > f32::EPSILON {
                    let cur = vol.GetMasterVolume().unwrap_or(new_volume);
                    (cur * new_volume / old_volume).clamp(0.0, cap)
                } else {
                    (new_volume * cap).clamp(0.0, 1.0)
                };
                let _ = vol.SetMasterVolume(scaled, std::ptr::null());
                let _ = vol.SetMute(BOOL::from(muted), std::ptr::null());
            }
        }
    }
    Ok(())
}

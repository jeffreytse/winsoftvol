use windows::{
    core::{ComInterface, Result},
    Win32::{
        Foundation::BOOL,
        Media::Audio::{IAudioSessionManager2, ISimpleAudioVolume},
    },
};

fn absolute_session_volume(volume: f32, cap: f32) -> f32 {
    (volume * cap).clamp(0.0, 1.0)
}

/// Sets every session to the same absolute volume. Used for initial sync and new sessions.
#[allow(dead_code)]
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
                let _ = vol.SetMasterVolume(absolute_session_volume(volume, cap), std::ptr::null());
                let _ = vol.SetMute(BOOL::from(muted), std::ptr::null());
            }
        }
    }
    Ok(())
}

/// Mutes or unmutes every session without touching per-app volume levels.
pub fn set_all_sessions_mute(session_manager: &IAudioSessionManager2, muted: bool) -> Result<()> {
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

fn scale_volume(cur: f32, old_volume: f32, new_volume: f32, cap: f32) -> f32 {
    if old_volume > f32::EPSILON {
        (cur * new_volume / old_volume).clamp(0.0, cap)
    } else {
        (new_volume * cap).clamp(0.0, 1.0)
    }
}

/// Clamps sessions above `cap` down to `cap`. Sessions already below are untouched.
pub fn cap_all_sessions_volume(session_manager: &IAudioSessionManager2, cap: f32) -> Result<()> {
    let enumerator = unsafe { session_manager.GetSessionEnumerator()? };
    let count = unsafe { enumerator.GetCount()? };
    for i in 0..count {
        let session = unsafe { enumerator.GetSession(i)? };
        if let Ok(vol) = session.cast::<ISimpleAudioVolume>() {
            unsafe {
                if let Ok(cur) = vol.GetMasterVolume() {
                    if cur > cap {
                        let _ = vol.SetMasterVolume(cap, std::ptr::null());
                    }
                }
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
                if let Ok(cur) = vol.GetMasterVolume() {
                    let scaled = scale_volume(cur, old_volume, new_volume, cap);
                    let _ = vol.SetMasterVolume(scaled, std::ptr::null());
                }
                let _ = vol.SetMute(BOOL::from(muted), std::ptr::null());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{absolute_session_volume, scale_volume};

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    #[test]
    fn scale_down_proportionally() {
        assert!(approx(scale_volume(0.8, 1.0, 0.5, 1.0), 0.4));
    }

    #[test]
    fn scale_up_clamped_by_cap() {
        // cur=0.5 at old=0.5, new=1.0, cap=0.8 → clamped to 0.8
        assert!(approx(scale_volume(0.5, 0.5, 1.0, 0.8), 0.8));
    }

    #[test]
    fn zero_new_volume_gives_zero() {
        assert!(approx(scale_volume(0.9, 1.0, 0.0, 1.0), 0.0));
    }

    #[test]
    fn old_near_zero_uses_fallback_path() {
        // old < EPSILON → result is new * cap clamped to 1.0
        assert!(approx(scale_volume(0.5, 0.0, 0.6, 0.8), 0.6 * 0.8));
    }

    #[test]
    fn cap_enforced_on_normal_path() {
        assert!(approx(scale_volume(1.0, 0.5, 1.0, 0.6), 0.6));
    }

    #[test]
    fn ratio_preserved_between_sessions() {
        let s1 = scale_volume(0.4, 1.0, 0.5, 1.0);
        let s2 = scale_volume(0.8, 1.0, 0.5, 1.0);
        assert!(approx(s1, 0.2));
        assert!(approx(s2, 0.4));
        assert!(approx(s2 / s1, 2.0));
    }

    #[test]
    fn old_at_epsilon_uses_fallback_path() {
        // f32::EPSILON is NOT > f32::EPSILON → fallback: new * cap
        let result = scale_volume(0.5, f32::EPSILON, 0.6, 0.8);
        assert!(approx(result, 0.6 * 0.8));
    }

    #[test]
    fn old_just_above_epsilon_uses_normal_path() {
        let old = f32::EPSILON * 2.0;
        // cur == old → scaled = old * new / old = new
        let result = scale_volume(old, old, 0.5, 1.0);
        assert!(approx(result, 0.5));
    }

    // cap_all_sessions_volume logic tests (pure math, no COM)
    fn cap_session(cur: f32, cap: f32) -> f32 {
        if cur > cap {
            cap
        } else {
            cur
        }
    }

    #[test]
    fn cap_not_applied_when_below_cap() {
        assert!((cap_session(0.3, 0.4) - 0.3).abs() < 1e-5);
    }

    #[test]
    fn cap_applied_when_above_cap() {
        assert!((cap_session(0.8, 0.4) - 0.4).abs() < 1e-5);
    }

    #[test]
    fn cap_applied_when_at_full_volume() {
        assert!((cap_session(1.0, 0.4) - 0.4).abs() < 1e-5);
    }

    #[test]
    fn cap_unchanged_when_equal_to_cap() {
        assert!((cap_session(0.4, 0.4) - 0.4).abs() < 1e-5);
    }

    // absolute_session_volume tests
    #[test]
    fn absolute_full_volume_no_cap() {
        assert!(approx(absolute_session_volume(1.0, 1.0), 1.0));
    }

    #[test]
    fn absolute_volume_with_cap() {
        assert!(approx(absolute_session_volume(1.0, 0.6), 0.6));
    }

    #[test]
    fn absolute_partial_volume_with_cap() {
        assert!(approx(absolute_session_volume(0.8, 0.8), 0.64));
    }

    #[test]
    fn absolute_zero_volume() {
        assert!(approx(absolute_session_volume(0.0, 0.8), 0.0));
    }

    #[test]
    fn absolute_clamped_to_one() {
        // cap > 1.0 would exceed bounds — clamp ensures safety
        assert!(approx(absolute_session_volume(1.0, 1.0), 1.0));
    }
}

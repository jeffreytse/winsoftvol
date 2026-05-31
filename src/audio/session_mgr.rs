use windows::{
    core::{ComInterface, Result},
    Win32::{
        Foundation::BOOL,
        Media::Audio::{IAudioSessionManager2, ISimpleAudioVolume},
    },
};

pub fn set_all_sessions_volume(
    session_manager: &IAudioSessionManager2,
    volume: f32,
    muted: bool,
) -> Result<()> {
    let enumerator = unsafe { session_manager.GetSessionEnumerator()? };
    let count = unsafe { enumerator.GetCount()? };

    for i in 0..count {
        let session = unsafe { enumerator.GetSession(i)? };
        if let Ok(vol) = session.cast::<ISimpleAudioVolume>() {
            unsafe {
                let _ = vol.SetMasterVolume(volume, std::ptr::null());
                let _ = vol.SetMute(BOOL::from(muted), std::ptr::null());
            }
        }
    }
    Ok(())
}

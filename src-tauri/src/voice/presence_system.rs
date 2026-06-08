use crate::consciousness::PresenceState;
use std::path::Path;
use std::ptr;

/// Plays the corresponding presence state background audio.
pub fn play_presence_signature(state: PresenceState) {
    // 1. Stop any currently playing asynchronous audio first
    unsafe {
        #[link(name = "winmm")]
        extern "system" {
            fn PlaySoundW(pszSound: *const u16, hmod: isize, fdwSound: u32) -> i32;
        }
        // SND_PURGE = 0x0040
        PlaySoundW(ptr::null(), 0, 0x0040);
    }

    // 2. Identify target presence audio path
    let state_name = match state {
        PresenceState::Sleeping => "sleeping",
        PresenceState::Listening => "listening",
        PresenceState::Thinking => "thinking",
        PresenceState::Speaking => "speaking",
        PresenceState::Working => "working",
        PresenceState::Monitoring => "monitoring",
    };

    let dir = "e:\\ALOK PC\\models\\presence";
    let wav_path = format!("{}\\{}.wav", dir, state_name);

    if !Path::new(&wav_path).exists() {
        println!("PresenceSystem: Signature file for {:?} not found at {}", state, wav_path);
        return;
    }

    println!("PresenceSystem: Playing audio signature for {:?}", state);

    // 3. Play asynchronously. Use loop for background hums (Sleeping, Thinking, Working, Monitoring)
    let should_loop = match state {
        PresenceState::Sleeping | PresenceState::Thinking | PresenceState::Working | PresenceState::Monitoring => true,
        PresenceState::Listening | PresenceState::Speaking => false,
    };

    let flags = if should_loop {
        // SND_FILENAME = 0x00020000, SND_ASYNC = 0x0001, SND_LOOP = 0x0008, SND_NODEFAULT = 0x0002
        0x00020000 | 0x0001 | 0x0008 | 0x0002
    } else {
        // SND_FILENAME = 0x00020000, SND_ASYNC = 0x0001, SND_NODEFAULT = 0x0002
        0x00020000 | 0x0001 | 0x0002
    };

    unsafe {
        #[link(name = "winmm")]
        extern "system" {
            fn PlaySoundW(pszSound: *const u16, hmod: isize, fdwSound: u32) -> i32;
        }
        let path_utf16: Vec<u16> = wav_path
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        PlaySoundW(path_utf16.as_ptr(), 0, flags);
    }
}

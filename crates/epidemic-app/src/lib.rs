mod audio;

/// Shared entry point — both desktop and Android call this.
pub fn run() {
    env_logger::init();

    // Start audio manager
    let audio = audio::AudioManager::new();
    if let Some(ref audio) = audio {
        audio.play_music();
    }

    epidemic_render::run();
}

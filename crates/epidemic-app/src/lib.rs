use std::io::BufReader;
use rodio::Source;

/// Shared entry point — both desktop and Android call this.
pub fn run() {
    env_logger::init();

    // Start background music in a separate thread
    std::thread::spawn(|| {
        play_background_music();
    });

    epidemic_render::run();
}

fn play_background_music() {
    let Ok((_stream, stream_handle)) = rodio::OutputStream::try_default() else {
        eprintln!("Could not open audio output");
        return;
    };

    // Try multiple paths for the music file
    let paths = [
        "../Assets/Overture.mp3",
        "Assets/Overture.mp3",
        "../assets/Overture.mp3",
        "assets/Overture.mp3",
    ];

    let mut file = None;
    for path in &paths {
        if let Ok(f) = std::fs::File::open(path) {
            file = Some(f);
            break;
        }
    }

    let Some(file) = file else {
        eprintln!("Could not find Overture.mp3");
        return;
    };

    let reader = BufReader::new(file);
    let Ok(source) = rodio::Decoder::new(reader) else {
        eprintln!("Could not decode audio file");
        return;
    };

    // Loop forever
    let source = source.repeat_infinite();

    match stream_handle.play_raw(source.convert_samples()) {
        Ok(()) => {
            // Keep the thread alive so audio keeps playing
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
        Err(e) => eprintln!("Audio playback error: {e}"),
    }
}

use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::io::BufReader;
use std::sync::{Arc, Mutex};

pub struct AudioManager {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    music_sink: Option<Sink>,
    sfx_sink: Option<Sink>,
}

impl AudioManager {
    pub fn new() -> Option<Self> {
        let (stream, stream_handle) = OutputStream::try_default().ok()?;
        let music_sink = Sink::try_new(&stream_handle).ok()?;
        let sfx_sink = Sink::try_new(&stream_handle).ok()?;

        Some(Self {
            _stream: stream,
            stream_handle,
            music_sink: Some(music_sink),
            sfx_sink: Some(sfx_sink),
        })
    }

    pub fn play_music(&self) {
        let paths = [
            "../Assets/Overture.mp3",
            "Assets/Overture.mp3",
            "../assets/Overture.mp3",
            "assets/Overture.mp3",
        ];

        for path in &paths {
            if let Ok(file) = std::fs::File::open(path) {
                let reader = BufReader::new(file);
                if let Ok(source) = rodio::Decoder::new(reader) {
                    let source = source.repeat_infinite();
                    if let Some(sink) = &self.music_sink {
                        sink.append(source);
                        sink.set_volume(0.3);
                    }
                    return;
                }
            }
        }
    }

    pub fn play_sfx(&self, _name: &str) {
        // Sound effects would go here when we have .wav/.ogg files
        // For now, we can use system beeps or short synthesized tones
    }

    pub fn set_music_volume(&self, volume: f32) {
        if let Some(sink) = &self.music_sink {
            sink.set_volume(volume);
        }
    }

    pub fn pause_music(&self) {
        if let Some(sink) = &self.music_sink {
            sink.pause();
        }
    }

    pub fn resume_music(&self) {
        if let Some(sink) = &self.music_sink {
            sink.play();
        }
    }
}

use anyhow::Result;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::collections::HashMap;
use std::io::{BufReader, Cursor};
use std::path::PathBuf;

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct EmbeddedAssets;

/// Represents a music track that can be played
#[derive(Debug, Clone)]
pub struct AudioTrack {
    path: PathBuf,
    volume: f32,
}

#[allow(dead_code)]
impl AudioTrack {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            path: path.into(),
            volume: 1.0,
        }
    }

    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }
}

/// Manages audio playback with support for music and sound effects
pub struct AudioManager {
    _stream: OutputStream,
    music_sink: Sink,
    sfx_sink: Sink,
    music_tracks: HashMap<String, AudioTrack>,
    sfx_tracks: HashMap<String, AudioTrack>,
}

#[allow(dead_code)]
impl AudioManager {
    fn read_embedded_bytes(path: &PathBuf) -> Option<Vec<u8>> {
        // Find the first occurrence of "assets/" in the path and use the suffix
        let path_str = path.to_string_lossy();
        if let Some(idx) = path_str.find("assets/") {
            let rel = &path_str[idx + "assets/".len()..];
            if let Some(f) = EmbeddedAssets::get(rel) {
                return Some(f.data.into_owned());
            }
        }
        None
    }
    /// Create a new AudioManager
    pub fn new() -> Result<Self> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let music_sink = Sink::try_new(&stream_handle)?;
        let sfx_sink = Sink::try_new(&stream_handle)?;

        Ok(Self {
            _stream,
            music_sink,
            sfx_sink,
            music_tracks: HashMap::new(),
            sfx_tracks: HashMap::new(),
        })
    }

    /// Register a new music track
    pub fn register_music<S: Into<String>>(&mut self, name: S, track: AudioTrack) {
        self.music_tracks.insert(name.into(), track);
    }

    /// Register a new sound effect
    pub fn register_sound_effect<S: Into<String>>(&mut self, name: S, track: AudioTrack) {
        self.sfx_tracks.insert(name.into(), track);
    }

    /// Play a music track (stops any currently playing music)
    pub fn play_music(&mut self, name: &str) -> Result<()> {
        if let Some(track) = self.music_tracks.get(name) {
            let bytes = Self::read_embedded_bytes(&track.path)
                .unwrap_or_else(|| panic!("Missing embedded audio: {:?}", track.path));
            let source = Decoder::new(BufReader::new(Cursor::new(bytes)))?;

            self.music_sink.stop();
            self.music_sink
                .append(source.repeat_infinite().amplify(track.volume));
        }
        Ok(())
    }

    /// Play a sound effect (can play simultaneously with music)
    pub fn play_sound_effect(&mut self, name: &str) -> Result<()> {
        if let Some(track) = self.sfx_tracks.get(name) {
            let bytes = Self::read_embedded_bytes(&track.path)
                .unwrap_or_else(|| panic!("Missing embedded audio: {:?}", track.path));
            let source = Decoder::new(BufReader::new(Cursor::new(bytes)))?;

            // Play the sound effect without stopping music
            self.sfx_sink.append(source.amplify(track.volume));
        }
        Ok(())
    }

    /// Stop all audio
    pub fn stop_all(&mut self) {
        self.music_sink.stop();
        self.sfx_sink.stop();
    }

    /// Stop only the music
    pub fn stop_music(&mut self) {
        self.music_sink.stop();
    }

    /// Stop all sound effects
    pub fn stop_sound_effects(&mut self) {
        self.sfx_sink.stop();
    }

    /// Set the music volume (0.0 to 1.0)
    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_sink.set_volume(volume.clamp(0.0, 1.0));
    }

    /// Set the sound effects volume (0.0 to 1.0)
    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_sink.set_volume(volume.clamp(0.0, 1.0));
    }
}

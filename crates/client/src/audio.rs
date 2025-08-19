use anyhow::Result;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

/// Represents a music track that can be played
#[derive(Debug, Clone)]
pub struct AudioTrack {
    path: PathBuf,
    volume: f32,
}

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

impl AudioManager {
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
            let file = File::open(&track.path)?;
            let source = Decoder::new(BufReader::new(file))?;
            
            self.music_sink.stop();
            self.music_sink.append(source.repeat_infinite().amplify(track.volume));
        }
        Ok(())
    }

    /// Play a sound effect (can play simultaneously with music)
    pub fn play_sound_effect(&mut self, name: &str) -> Result<()> {
        if let Some(track) = self.sfx_tracks.get(name) {
            let file = File::open(&track.path)?;
            let source = Decoder::new(BufReader::new(file))?;
            
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

    /// Test function to verify audio playback works
    pub fn test_audio_playback() -> Result<()> {
        println!("Testing audio playback...");
        
        // Test if we can create an output stream
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| anyhow::anyhow!("Failed to create output stream: {}", e))?;
        println!("✓ Output stream created successfully");
        
        // Test if we can create a sink
        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| anyhow::anyhow!("Failed to create sink: {}", e))?;
        println!("✓ Sink created successfully");
        
        // Test if we can load and play a sound
        let test_file = "crates/client/assets/sound/music/opening.mp3";
        println!("Trying to load: {}", test_file);
        
        let file = File::open(test_file)
            .map_err(|e| anyhow::anyhow!("Failed to open file: {}", e))?;
        println!("✓ File opened successfully");
        
        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| anyhow::anyhow!("Failed to decode audio: {}", e))?;
        println!("✓ Audio decoded successfully");
        
        println!("Playing test sound...");
        sink.append(source);
        
        // Wait for the sound to finish
        std::thread::sleep(std::time::Duration::from_secs(5));
        
        println!("Audio test completed");
        Ok(())
    }

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_audio_playback() -> Result<()> {
        let mut audio = AudioManager::new()?;
        
        // Register tracks
        audio.register_track("opening", AudioTrack::new("assets/sound/music/opening.mp3").with_volume(0.5));
        audio.register_track("combat", AudioTrack::new("assets/sound/music/combat.mp3").with_volume(0.6));
        
        // Test playing tracks with transitions
        audio.play_track("opening")?;
        std::thread::sleep(Duration::from_secs(2));
        
        // Test transition to combat music
        audio.play_track("combat")?;
        std::thread::sleep(Duration::from_secs(2));
        
        // Test volume adjustment
        audio.set_track_volume("combat", 0.3)?;
        std::thread::sleep(Duration::from_secs(1));
        
        // Test stopping
        audio.stop();
        
        Ok(())
    }
}

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

/// Manages audio playback with support for multiple tracks
pub struct AudioManager {
    _stream: OutputStream,
    sink: Sink,
    tracks: HashMap<String, AudioTrack>,
}

impl AudioManager {
    /// Create a new AudioManager
    pub fn new() -> Result<Self> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        
        Ok(Self {
            _stream,
            sink,
            tracks: HashMap::new(),
        })
    }

    /// Register a new audio track with a given name
    pub fn register_track<S: Into<String>>(&mut self, name: S, track: AudioTrack) {
        self.tracks.insert(name.into(), track);
    }

    /// Play a registered track by name
    pub fn play_track(&mut self, name: &str) -> Result<()> {
        if let Some(track) = self.tracks.get(name) {
            let file = File::open(&track.path)?;
            let source = Decoder::new(BufReader::new(file))?;
            
            // Stop any currently playing track
            self.sink.stop();
            
            // Play the new track
            self.sink.append(source.repeat_infinite().amplify(track.volume));
        }
        
        Ok(())
    }

    /// Stop the currently playing track
    pub fn stop(&mut self) {
        self.sink.stop();
    }

    /// Set the volume for a specific track
    pub fn set_track_volume(&mut self, name: &str, volume: f32) -> Result<()> {
        if let Some(track) = self.tracks.get_mut(name) {
            track.volume = volume.clamp(0.0, 1.0);
            self.sink.set_volume(volume);
        }
        Ok(())
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

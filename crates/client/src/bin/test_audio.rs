use anyhow::Result;
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;

fn test_audio_playback() -> Result<()> {
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
    println!("Audio should be playing now. Will wait for 5 seconds...");
    std::thread::sleep(std::time::Duration::from_secs(5));
    
    println!("Audio test completed");
    Ok(())
}

fn main() -> Result<()> {
    println!("Starting audio test...");
    
    // First, try the basic audio test
    if let Err(e) = test_audio_playback() {
        eprintln!("Audio test failed: {}", e);
        return Err(e);
    }
    
    println!("All tests completed successfully!");
    
    // Keep the program alive to hear the audio
    println!("Press Ctrl+C to exit...");
    std::thread::sleep(std::time::Duration::from_secs(10));
    
    Ok(())
}

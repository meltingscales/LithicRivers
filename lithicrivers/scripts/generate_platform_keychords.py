#!/usr/bin/env python3
"""
Generate platform-specific keychord mappings by prompting user input.

This script prompts the user to type specific key combinations and captures
the keycode sequences. Run this once per platform to create keychord mappings.

Usage:
    python generate_platform_keychords.py
"""

import json
import sys
import platform
import logging
from pathlib import Path
from typing import Dict, List, Optional

try:
    from asciimatics.screen import Screen
    from asciimatics.event import KeyboardEvent
except ImportError:
    print("Error: asciimatics not installed")
    print("Install with: pip install asciimatics")
    sys.exit(1)

# Set up file-based logging
logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('keychord_capture.log'),
    ]
)
logger = logging.getLogger(__name__)

TESTING_LIMIT_CAPTURE=False

class KeychordCapture:
    def __init__(self):
        self.captured_keychords: Dict[str, List[int]] = {}
        self.current_sequence: List[int] = []
        self.recording = False
        self.current_prompt = ""
        self._load_existing_keychords()
        
    def get_keychord_prompts(self) -> List[str]:
        """Define the key combinations to capture."""
        prompts = []
        
        # Uppercase letters
        for char in "ABCDEFGHIJKLMNOPQRSTUVWXYZ":
            prompts.append(char)
        
        # Lowercase letters  
        for char in "abcdefghijklmnopqrstuvwxyz":
            prompts.append(char)
        
        # Numbers
        for char in "0123456789":
            prompts.append(char)
        
        # Function keys
        for i in range(1, 13):
            prompts.append(f"F{i}")
        
        # Common symbols
        symbols = "!@#$%^&*()_+-=[]{}|\\;:'\",./<>?"
        for char in symbols:
            prompts.append(char)
        
        # Special keys
        special_keys = [
            "ESCAPE", "ENTER", "BACKSPACE", "TAB", "SPACE",
            "UP_ARROW", "DOWN_ARROW", "LEFT_ARROW", "RIGHT_ARROW",
            "HOME", "END", "PAGE_UP", "PAGE_DOWN", "INSERT", "DELETE"
        ]
        prompts.extend(special_keys)
        
        # Numpad keys
        numpad_keys = [
            "NUMPAD_0", "NUMPAD_1", "NUMPAD_2", "NUMPAD_3", "NUMPAD_4",
            "NUMPAD_5", "NUMPAD_6", "NUMPAD_7", "NUMPAD_8", "NUMPAD_9",
            "NUMPAD_PLUS", "NUMPAD_MINUS", "NUMPAD_MULTIPLY", "NUMPAD_DIVIDE",
            "NUMPAD_ENTER", "NUMPAD_DECIMAL"
        ]
        prompts.extend(numpad_keys)
        
        return prompts
    
    def _get_platform_info(self):
        """Get platform name and OS type for filename."""
        platform_name = platform.system().lower()
        
        # Detect OS type
        os_type = "unknown"
        try:
            with open("/etc/os-release", "r") as f:
                for line in f:
                    if line.startswith("ID="):
                        os_type = line.split("=")[1].strip().strip('"')
                        break
        except FileNotFoundError:
            # Fallback for systems without /etc/os-release
            if platform_name == "linux":
                os_type = "linux"
            elif platform_name == "darwin":
                os_type = "macos"
            elif platform_name == "windows":
                os_type = "windows"
        
        return platform_name, os_type
    
    def _load_existing_keychords(self):
        """Load existing keychords from the platform-specific JSON file."""
        platform_name, os_type = self._get_platform_info()
        filename = f"keychords.{platform_name}.{os_type}.json"
        config_dir = Path("config")
        existing_file = config_dir / filename
        
        if existing_file.exists():
            try:
                with open(existing_file, 'r') as f:
                    existing_data = json.load(f)
                    self.captured_keychords.update(existing_data)
                    logger.info(f"📂 Loaded {len(existing_data)} existing keychords from {existing_file}")
            except Exception as e:
                logger.warning(f"⚠️ Could not load existing keychords from {existing_file}: {e}")
        else:
            logger.info(f"📂 No existing keychord file found at {existing_file}")
    
    def start_capture(self, screen: Screen):
        """Start the interactive capture process."""
        logger.info("🎯 start_capture() called!")
        
        prompts = self.get_keychord_prompts()
        total_prompts = len(prompts)
        
        logger.info(f"📋 Got {total_prompts} prompts to capture")
        
        screen.clear()
        screen.print_at("Platform Keychord Capture Tool", 0, 0)
        screen.print_at(f"Platform: {platform.system()} {platform.release()}", 0, 1)
        screen.print_at(f"Total keys to capture: {total_prompts}", 0, 2)
        screen.print_at("Press any key to start...", 0, 3)
        screen.refresh()
        
        # Wait for start
        while True:
            event = screen.get_event()
            if event and hasattr(event, 'key_code'):
                break
        
        # Capture each keychord
        captured_count = 0
        skipped_count = 0
        
        for i, prompt in enumerate(prompts):
            # Skip if already captured (but allow re-capturing if marked as not working)
            if prompt in self.captured_keychords and self.captured_keychords[prompt] is not None:
                skipped_count += 1
                logger.info(f"⏭️ Skipping '{prompt}' - already captured as {self.captured_keychords[prompt]}")
                continue
                
            if not self._capture_single_keychord(screen, prompt, i + 1 - skipped_count, total_prompts - skipped_count):
                break

            captured_count += 1

            # for testing - remove later.
            if(TESTING_LIMIT_CAPTURE and (captured_count == 3)):
                logger.info("🧪 Test mode: Breaking after 3 keys")
                break
        
        # Save results
        logger.info(f"💾 Saving {len(self.captured_keychords)} captured keychords...")
        logger.info(f"📊 Captured: {captured_count}, Skipped: {skipped_count}")
        self._save_results()
    
    def _capture_single_keychord(self, screen: Screen, prompt: str, current: int, total: int) -> bool:
        """Capture a single keychord."""
        screen.clear()
        
        # Display progress and instructions
        screen.print_at(f"Progress: {current}/{total}", 0, 0)
        
        # Create progress bar
        progress_width = 40
        progress_filled = int((current / total) * progress_width)
        progress_bar = "█" * progress_filled + "░" * (progress_width - progress_filled)
        screen.print_at(f"Progress: [{progress_bar}] {current}/{total}", 0, 1)
        
        screen.print_at(f"Please press: {prompt}", 0, 3)
        screen.print_at("Press the key combination now...", 0, 4)
        screen.print_at("(Press ESC to add the current sequence to the keychord)", 0, 5)
        screen.print_at("(Press ESC at start to exit)", 0, 6)
        screen.print_at("💡 If a key doesn't work, press ESC to skip, then edit the JSON manually", 0, 7)
        
        # Add capitalization hint for uppercase letters
        if prompt.isupper() and len(prompt) == 1:
            screen.print_at("💡 Hint: Press SHIFT + the letter for uppercase", 0, 8)
        
        # Show previously captured sequences
        if self.captured_keychords:
            screen.print_at("Recently captured:", 0, 10)
            y_offset = 11
            for key, sequence in list(self.captured_keychords.items())[-5:]:
                screen.print_at(f"  {key}: {sequence}", 0, y_offset)
                y_offset += 1
        
        screen.refresh()
        
        # Capture the keychord
        sequence = self._capture_sequence(screen)
        
        if sequence is None:
            return False  # Exit
        elif sequence == []:
            return True   # Skip this key
        
        # Store the result
        self.captured_keychords[prompt] = sequence
        
        # Show confirmation
        screen.print_at(f"Captured: {prompt} -> {sequence}", 0, 15)
        screen.refresh()
        
        return True
    
    def _capture_sequence(self, screen: Screen) -> Optional[List[int]]:
        """Capture a key sequence from user input."""
        sequence = []
        
        while True:
            event = screen.get_event()
            
            if event and hasattr(event, 'key_code'):
                keycode = event.key_code
                
                # Handle special cases
                if keycode == 27 or keycode == -1:  # ESC (different terminals use different codes)
                    if sequence:  # ESC pressed during sequence, meaning we want to use the current sequence    
                        return sequence  # Return the current sequence
                    else:  # ESC pressed at start
                        return None  # Exit
                
                # Add to sequence
                sequence.append(keycode)
                
                # Display current sequence
                screen.print_at(f"Sequence: {sequence}", 0, 10)
                screen.print_at("Press another key to add it, or press ESC to finish...", 0, 11)
                screen.refresh()
            else:
                # No event, just continue waiting
                pass
    
    def _save_results(self):
        """Save captured keychords to JSON file."""
        logger.info(f"🔍 Debug: _save_results() called with {len(self.captured_keychords)} keychords")
        
        if not self.captured_keychords:
            logger.error("❌ No keychords captured.")
            return
        
        # Create filename based on platform and OS type
        platform_name, os_type = self._get_platform_info()
        filename = f"keychords.{platform_name}.{os_type}.json"
        
        # Save to config directory
        config_dir = Path("config")
        config_dir.mkdir(exist_ok=True)
        output_path = config_dir / filename
        
        logger.info(f"📁 Saving to: {output_path}")
        
        try:
            with open(output_path, 'w') as f:
                json.dump(self.captured_keychords, f, indent=2, sort_keys=True)
            
            logger.info(f"✅ Keychords saved to: {output_path}")
            logger.info(f"📊 Total keychords captured: {len(self.captured_keychords)}")
            
            # Show summary
            logger.info("📋 Captured keychords:")
            for key, sequence in sorted(self.captured_keychords.items()):
                logger.info(f"  {key}: {sequence}")
        except Exception as e:
            logger.error(f"❌ Error saving keychords: {e}")


def main():
    """Main entry point."""
    logger.info("Platform Keychord Capture Tool")
    logger.info("=" * 40)
    
    # Get platform info
    platform_name = platform.system().lower()
    try:
        with open("/etc/os-release", "r") as f:
            for line in f:
                if line.startswith("ID="):
                    os_type = line.split("=")[1].strip().strip('"')
                    break
            else:
                os_type = "unknown"
    except FileNotFoundError:
        os_type = "unknown"
    
    logger.info(f"Platform: {platform.system()} {platform.release()}")
    logger.info(f"OS Type: {os_type}")
    logger.info("This tool will prompt you to press various keys and key combinations.")
    logger.info("The results will be saved to a platform-specific JSON file.")
    logger.info("")
    logger.info("💡 For keys that don't work (like F11), press ESC to skip them.")
    logger.info("   Then manually edit the JSON file to add: \"F11\": null")
    logger.info("")
    
    try:
        logger.info("🔧 Creating KeychordCapture instance...")
        capture = KeychordCapture()
        logger.info("🎮 Starting asciimatics screen wrapper...")
        
        # Create a wrapper function that calls our capture method
        def demo(screen):
            logger.info("🎯 Demo function called!")
            capture.start_capture(screen)
        
        Screen.wrapper(demo)
        logger.info("✅ Screen wrapper completed.")
    except KeyboardInterrupt:
        logger.info("Capture interrupted by user.")
    except Exception as e:
        logger.error(f"❌ Error during capture: {e}")
        import traceback
        traceback.print_exc()
        return 1
    
    return 0


if __name__ == "__main__":
    sys.exit(main()) 
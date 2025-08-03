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
from pathlib import Path
from typing import Dict, List, Optional

try:
    from asciimatics.screen import Screen
    from asciimatics.event import KeyboardEvent
except ImportError:
    print("Error: asciimatics not installed")
    print("Install with: pip install asciimatics")
    sys.exit(1)


class KeychordCapture:
    def __init__(self):
        self.captured_keychords: Dict[str, List[int]] = {}
        self.current_sequence: List[int] = []
        self.recording = False
        self.current_prompt = ""
        
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
    
    def start_capture(self, screen: Screen):
        """Start the interactive capture process."""
        prompts = self.get_keychord_prompts()
        total_prompts = len(prompts)
        
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
        for i, prompt in enumerate(prompts):
            if not self._capture_single_keychord(screen, prompt, i + 1, total_prompts):
                break

            # for testing - remove later.
            if(i == 3):
                break
        
        # Save results
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
        screen.print_at("(Press ESC to skip this key)", 0, 5)
        screen.print_at("(Press ESC at start to exit)", 0, 6)
        
        # Add capitalization hint for uppercase letters
        if prompt.isupper() and len(prompt) == 1:
            screen.print_at("💡 Hint: Press SHIFT + the letter for uppercase", 0, 7)
        
        # Show previously captured sequences
        if self.captured_keychords:
            screen.print_at("Recently captured:", 0, 9)
            y_offset = 10
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
        screen.print_at("Press any key to continue...", 0, 16)
        screen.refresh()
        
        # Wait for confirmation
        while True:
            event = screen.get_event()
            if event and hasattr(event, 'key_code'):
                break
        
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
                    if sequence:  # ESC pressed during sequence
                        return []  # Skip this key
                    else:  # ESC pressed at start
                        return None  # Exit
                
                # Add to sequence
                sequence.append(keycode)
                
                # Display current sequence
                screen.print_at(f"Sequence: {sequence}", 0, 10)
                screen.print_at("Press another key to add it, or press ESC (and wait) to finish...", 0, 11)
                screen.refresh()
            else:
                # No event, just continue waiting
                pass
    
    def _save_results(self):
        """Save captured keychords to JSON file."""
        if not self.captured_keychords:
            print("No keychords captured.")
            return
        
        # Create filename based on platform
        platform_name = platform.system().lower()
        filename = f"keychords.{platform_name}.json"
        
        # Save to config directory
        config_dir = Path("config")
        config_dir.mkdir(exist_ok=True)
        output_path = config_dir / filename
        
        with open(output_path, 'w') as f:
            json.dump(self.captured_keychords, f, indent=2, sort_keys=True)
        
        print(f"\nKeychords saved to: {output_path}")
        print(f"Total keychords captured: {len(self.captured_keychords)}")
        
        # Show summary
        print("\nCaptured keychords:")
        for key, sequence in sorted(self.captured_keychords.items()):
            print(f"  {key}: {sequence}")


def main():
    """Main entry point."""
    print("Platform Keychord Capture Tool")
    print("=" * 40)
    print(f"Platform: {platform.system()} {platform.release()}")
    print("This tool will prompt you to press various keys and key combinations.")
    print("The results will be saved to a platform-specific JSON file.")
    print()
    
    try:
        capture = KeychordCapture()
        Screen.wrapper(capture.start_capture)
    except KeyboardInterrupt:
        print("\nCapture interrupted by user.")
    except Exception as e:
        print(f"\nError during capture: {e}")
        return 1
    
    return 0


if __name__ == "__main__":
    sys.exit(main()) 
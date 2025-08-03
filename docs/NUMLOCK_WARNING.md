# NumLock Warning Feature

## Overview

The game now includes a direct NumLock state detection system that checks the actual numlock state during game boot and automatically detects when NumLock is toggled on.

## How It Works

### Direct State Detection
The system uses platform-specific APIs to directly query the numlock state:

1. **Windows**: Uses the Windows API (`GetKeyState`) to check numlock state
2. **Linux**: Uses the `xset q` command to check numlock state
3. **Other platforms**: Assumes numlock is on (graceful fallback)

### Detection Logic

**Platform-Specific Detection:**

**Windows:**
```python
import ctypes
hllDll = ctypes.WinDLL("User32.dll")
VK_NUMLOCK = 0x90
return bool(hllDll.GetKeyState(VK_NUMLOCK) & 0x0001)
```

**Linux:**
```python
import evdev
from evdev import ecodes
import subprocess
import os

# Try xset first (most portable)
result = subprocess.run(["xset", "q"], capture_output=True, text=True, timeout=1)
if result.returncode == 0:
    output = result.stdout.lower()
    import re
    if re.search(r"num lock:\s*on", output):
        return True
    elif re.search(r"num lock:\s*off", output):
        return False

# Check for headless environment
if not os.environ.get('DISPLAY'):
    return True  # Assume numlock is on in headless

# Try setleds as fallback
result = subprocess.run(["setleds", "-L"], capture_output=True, text=True, timeout=1)
            if result.returncode == 0:
                output = result.stdout.lower()
                import re
                if re.search(r"num lock:\s*on", output):
                    return True
                elif re.search(r"num lock:\s*off", output):
                    return False

# Safe default
return True
```

### User Experience

1. **Game Boot**: System checks numlock state immediately
2. **Warning Display**: If numlock is off, warning popup appears immediately
3. **Automatic Detection**: When player toggles numlock on, popup closes automatically
4. **Fallback Detection**: Still monitors for incorrect key presses as backup

### Warning Message

When NumLock is detected as off, the player sees:

```
⚠️  NUMLOCK WARNING ⚠️

Your NumLock key appears to be turned OFF. This can cause issues with movement controls.

When NumLock is OFF:
• Numpad 8 becomes Up Arrow
• Numpad 2 becomes Down Arrow  
• Numpad 4 becomes Left Arrow
• Numpad 6 becomes Right Arrow
• And so on...

To fix this:
1. Press your NumLock key to turn it ON
2. The numpad keys should then work normally for movement

You can still use Q/E for up/down movement regardless of NumLock state.
```

## Implementation Details

### Functions

- `get_numlock_state() -> bool`: Main function that detects numlock state using platform-specific APIs
- `_get_numlock_state_windows() -> bool`: Windows-specific detection using Windows API
- `_get_numlock_state_linux() -> bool`: Linux-specific detection using xset command
- `_detect_numlock_issue(event: KeyboardEvent) -> bool`: Fallback detection for incorrect key codes
- `_show_numlock_warning(world_map)`: Shows the warning popup

### Global State

- `numlock_warning_shown`: Tracks whether the warning popup has been shown
- `active_popup`: Tracks the currently active popup for automatic closing

### Integration

The system is integrated into the main event handler in `demo()` function:

1. **Boot-time check**: Checks numlock state during game initialization
2. **Immediate warning**: Shows warning popup if numlock is off
3. **Automatic closing**: Closes popup when numlock state changes to on
4. **Fallback detection**: Still monitors for incorrect key presses

### Platform Support

- **Windows**: Full support using Windows API
- **Linux**: Full support using xset/setleds (portable methods)
- **macOS**: Graceful fallback (assumes numlock is on)
- **Other platforms**: Graceful fallback (assumes numlock is on)
- **PyInstaller binaries**: Fully compatible across all platforms

## Testing

The feature includes comprehensive tests in `test_numlock_warning.py` that verify:

- Correct detection of NumLock off states
- Correct detection of NumLock on states  
- Platform-specific detection (Windows, Linux, other platforms)
- Exception handling for failed detection
- Proper handling of other key codes
- No false positives for normal keys

## Benefits

1. **Direct Detection**: Uses actual system APIs instead of inferring from key presses
2. **Immediate Feedback**: Shows warning during game boot if numlock is off
3. **Automatic Recovery**: Closes popup when numlock is toggled on
4. **Platform Agnostic**: Works on Windows, Linux, and other platforms
5. **Robust**: Graceful fallback if detection fails
6. **Non-intrusive**: Only shows once per session
7. **Educational**: Explains what's happening and how to fix it
8. **Deterministic**: Follows the game's principle of being fully deterministic

## Technical Notes

- Uses `ctypes` for Windows API access
- Uses `xset q` for Linux detection (primary method) with regex pattern matching for variable spacing
- Falls back to `setleds -L` command as secondary method
- Detects headless environments and assumes safe defaults
- Includes timeout protection for subprocess calls
- Graceful exception handling for all platform-specific code
- Assumes numlock is on if detection fails (safe default)
- **PyInstaller compatible**: No external dependencies required 
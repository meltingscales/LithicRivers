#!/usr/bin/env python3

"""
Intro screen that prints the boot message with a glitch effect.

Uses asciimatics to render each line slowly, applying
`lithicrivers.textutil.corrupt_text` over several frames before
settling on the final clean text.
"""

from pathlib import Path
from collections import deque
import sys

from asciimatics.effects import Print
from asciimatics.renderers import StaticRenderer
from asciimatics.scene import Scene
from asciimatics.screen import Screen
from asciimatics.exceptions import ResizeScreenError

from lithicrivers.textutil import corrupt_text


def _read_boot_lines() -> list[str]:
    """Read config/boot_message.dat assuming CWD is repo root (via Makefile)."""
    boot_path = Path("config") / "boot_message.dat"
    text = boot_path.read_text(encoding="utf-8")
    # Preserve empty lines; strip trailing newlines but not internal spacing
    return text.splitlines()


def _build_glitch_frames(line: str, glitch_frames: int = 20) -> list[str]:
    """Create a sequence of strings from corrupted to clean for one line."""
    frames = []
    # Start more heavily corrupted, end clean.
    for i in range(glitch_frames):
        # Linear falloff
        rate = 0.05 * (1.0 - (i / max(1, glitch_frames))) + 0.02
        frames.append(corrupt_text(line, corruption_rate=rate))
    frames.append(line)  # Final clean line
    return frames


def _intro(screen: Screen) -> None:
    lines = _read_boot_lines()

    # Layout parameters
    top_margin = max(1, screen.height // 6)
    left_margin = max(2, screen.width // 12)
    # Viewport height available for lines
    viewport_height = max(1, screen.height - top_margin - 2)

    # Timing parameters
    glitch_frames = 10

    # Build frame-by-frame images for a scrolling log window using a deque
    available_width = max(1, screen.width - left_margin - 1)
    window: deque[str] = deque(maxlen=viewport_height)
    images: list[str] = []

    for raw in lines:
        base_line = (raw or "")[:available_width]
        # Animate corrupted -> clean for this new line while it sits at the bottom
        for frame in _build_glitch_frames(base_line, glitch_frames=glitch_frames):
            # Push a temporary frame variant at the bottom without committing clean text yet
            window.append(base_line if len(window) == window.maxlen else base_line)
            # Replace the last line display with the current frame variant
            buffer_lines = list(window)
            if buffer_lines:
                buffer_lines[-1] = frame
            # Right-pad to fully overwrite previous characters and preserve whitespace
            padded = [l.ljust(available_width) for l in buffer_lines]
            images.append("\n".join(padded))
            # Remove the temp line so next frame can reuse previous state
            if window and (len(window) == viewport_height or True):
                window.pop()
        # Commit the clean line and let it remain in the window
        window.append(base_line)
        images.append("\n".join([l.ljust(available_width) for l in window]))

    # After all lines, keep a few idle frames to let the user read
    idle_tail = int(getattr(screen, "frame_rate", 30) * 1.5)
    if images:
        last_image = images[-1]
        images.extend([last_image] * idle_tail)

    # Use a single Print with a StaticRenderer that advances through images
    renderer = StaticRenderer(images=images)
    effect = Print(
        screen,
        renderer,
        y=top_margin,
        x=left_margin,
        start_frame=0,
        speed=1,
        transparent=False,
    )

    duration = max(len(images), 1)
    scene = Scene([effect], duration=duration, clear=True)
    screen.play([scene], stop_on_resize=True)


if __name__ == "__main__":
    # Run once and exit (no loop).
    try:
        Screen.wrapper(_intro)
    except ResizeScreenError:
        # Exit on resize instead of looping
        pass
    sys.exit(0)
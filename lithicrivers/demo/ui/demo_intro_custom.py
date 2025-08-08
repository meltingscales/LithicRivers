#!/usr/bin/env python3

"""
Intro screen that prints the boot message with a glitch effect.

Uses asciimatics to render each line slowly, applying
`lithicrivers.textutil.corrupt_text` over several frames before
settling on the final clean text.
"""

from pathlib import Path
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
        # Linear falloff from 0.25 -> ~0.02
        rate = 0.25 * (1.0 - (i / max(1, glitch_frames))) + 0.02
        frames.append(corrupt_text(line, corruption_rate=rate))
    frames.append(line)  # Final clean line
    return frames


def _intro(screen: Screen) -> None:
    lines = _read_boot_lines()

    # Layout parameters
    top_margin = max(1, screen.height // 6)
    left_margin = max(2, screen.width // 12)
    line_spacing = 1  # one row per line

    # Timing parameters
    glitch_frames = 18
    per_line_duration = glitch_frames + 4  # frames each line animates
    stagger = glitch_frames // 2  # how many frames between line starts

    effects = []
    for idx, raw_line in enumerate(lines):
        # Ensure nothing overflows the screen width; truncate if necessary.
        available_width = max(1, screen.width - left_margin - 1)
        line = raw_line[:available_width]

        y = top_margin + idx * line_spacing
        if y >= screen.height - 1:
            break  # Don't draw beyond screen

        frames = _build_glitch_frames(line, glitch_frames=glitch_frames)
        renderer = StaticRenderer(images=frames)

        start = idx * max(1, stagger)
        effects.append(
            Print(
                screen,
                renderer,
                y,
                x=left_margin,
                start_frame=start,
                speed=1,
                transparent=True,
            )
        )

    # Scene duration should cover all starts plus per-line animation
    if effects:
        last_start = (len(effects) - 1) * max(1, stagger)
        duration = last_start + per_line_duration
    else:
        duration = 60

    scene = Scene(effects, duration=duration, clear=True)
    screen.play([scene], stop_on_resize=True)


if __name__ == "__main__":
    # Run until we get a clean render without needing a resize.
    while True:
        try:
            Screen.wrapper(_intro)
            sys.exit(0)
        except ResizeScreenError:
            # Try again after resize
            pass
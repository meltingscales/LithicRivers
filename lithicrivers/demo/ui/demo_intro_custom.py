#!/usr/bin/env python3

"""
Intro screen that prints the boot message with a glitch effect.

Uses asciimatics to render each line slowly, applying
`lithicrivers.textutil.corrupt_text` over several frames before
settling on the final clean text.
"""

GLITCH_FRAMES = 5  # Number of glitch frames for intro animation

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


def _build_glitch_frames(line: str, glitch_frames: int) -> list[str]:
    """Create a sequence of strings from corrupted to clean for one line."""
    frames = []
    # Start more heavily corrupted, end clean.
    for i in range(glitch_frames):
        # Linear falloff
        rate = 0.50 * (1.0 - (i / max(1, glitch_frames))) + 0.02
        frames.append(corrupt_text(line, corruption_rate=rate))
    frames.append(line)  # Final clean line
    return frames


def _intro_scene(screen: Screen) -> Scene:
    lines = _read_boot_lines()
    top_margin = max(1, screen.height // 6)
    left_margin = max(2, screen.width // 12)
    viewport_height = max(1, screen.height - top_margin - 2)
    available_width = max(1, screen.width - left_margin - 1)
    window: deque[str] = deque(maxlen=viewport_height)
    images: list[str] = []
    for raw in lines:
        base_line = (raw or "")[:available_width]
        for frame in _build_glitch_frames(base_line, glitch_frames=GLITCH_FRAMES):
            window.append(base_line if len(window) == window.maxlen else base_line)
            buffer_lines = list(window)
            if buffer_lines:
                buffer_lines[-1] = frame
            padded = [l.ljust(available_width) for l in buffer_lines]
            images.append("\n".join(padded))
            if window and (len(window) == viewport_height or True):
                window.pop()
        window.append(base_line)
        images.append("\n".join([l.ljust(available_width) for l in window]))
    idle_tail = int(getattr(screen, "frame_rate", 30) * 1.5)
    if images:
        last_image = images[-1]
        images.extend([last_image] * idle_tail)
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
    return Scene([effect], duration=duration, clear=True)


def _dummy_world_scene(screen: Screen) -> Scene:
    world = "XXX\nXXX\nXXX"
    lines = world.split("\n")
    world_height = len(lines)
    world_width = max(len(line) for line in lines)
    y = max(0, (screen.height - world_height) // 2)
    x = max(0, (screen.width - world_width) // 2)
    renderer = StaticRenderer(images=[world])
    effect = Print(
        screen,
        renderer,
        y=y,
        x=x,
        start_frame=0,
        speed=0,
        transparent=False,
    )
    return Scene([effect], duration=-1, clear=True)


def main_demo(screen: Screen) -> None:
    scenes = [
        _intro_scene(screen),
        _dummy_world_scene(screen),
    ]
    screen.play(scenes, stop_on_resize=True)


if __name__ == "__main__":
    try:
        Screen.wrapper(main_demo)
    except ResizeScreenError:
        pass
    sys.exit(0)
#!/usr/bin/env python3
"""
Demo that reads config/story.txt and displays each word, one at a time, centered.
"""
from pathlib import Path
import sys
from collections import deque
from asciimatics.effects import Print
from asciimatics.renderers import StaticRenderer
from asciimatics.scene import Scene
from asciimatics.screen import Screen
from asciimatics.exceptions import ResizeScreenError

def _read_story_tokens() -> list[str]:
    story_path = Path("config") / "story.txt"
    text = story_path.read_text(encoding="utf-8")
    # Split into tokens: words and newlines as separate tokens
    tokens = []
    for line in text.splitlines(keepends=True):
        if line.strip() == "":
            tokens.append("\n")
        else:
            # Split line into words, preserving the newline
            words = line.strip().split()
            for word in words:
                tokens.append(word)
            if line.endswith("\n") or line.endswith("\r\n"):
                tokens.append("\n")
    # Remove trailing newline if present
    if tokens and tokens[-1] == "\n":
        tokens.pop()
    return tokens

def _story_scene(screen: Screen) -> Scene:
    tokens = _read_story_tokens()
    top_margin = max(1, screen.height // 6)
    left_margin = max(2, screen.width // 12)
    viewport_height = max(1, screen.height - top_margin - 2)
    available_width = max(1, screen.width - left_margin - 1)
    window: deque[str] = deque(maxlen=viewport_height)
    images: list[str] = []
    # For word wrapping
    built_lines: list[str] = [""]
    for i, token in enumerate(tokens):
        if token == "\n":
            built_lines.append("")
        else:
            current_line = built_lines[-1]
            if current_line:
                test_line = current_line + " " + token
            else:
                test_line = token
            if len(test_line) > available_width:
                built_lines.append(token)
            else:
                if current_line:
                    built_lines[-1] = current_line + " " + token
                else:
                    built_lines[-1] = token
        # For each frame, update the scroll window with the latest lines
        window.clear()
        for l in built_lines:
            # Truncate to available_width, pad to fill
            window.append(l[:available_width])
            if len(window) > viewport_height:
                window.popleft()
        padded = [l.ljust(available_width) for l in window]
        images.append("\n".join(padded))
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

def main_story_demo(screen: Screen) -> None:
    scenes = [_story_scene(screen)]
    screen.play(scenes, stop_on_resize=True)

if __name__ == "__main__":
    try:
        Screen.wrapper(main_story_demo)
    except ResizeScreenError:
        pass
    sys.exit(0)

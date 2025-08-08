#!/usr/bin/env python3
"""
Demo that reads config/story.txt and displays each word, one at a time, centered.
"""
from pathlib import Path
import sys
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
    # Build up the output, honoring newlines
    images = []
    lines = [""]
    for token in tokens:
        if token == "\n":
            lines.append("")
        else:
            if lines[-1]:
                lines[-1] += " " + token
            else:
                lines[-1] = token
        # Join all lines for the current frame
        img = "\n".join(lines)
        # Pad to screen height
        img += "\n" * (screen.height - img.count("\n") - 1)
        images.append(img)
    renderer = StaticRenderer(images=images)
    effect = Print(
        screen,
        renderer,
        y=0,
        x=0,
        start_frame=0,
        speed=1,
        transparent=False,
    )
    duration = max(len(images), 1)
    return Scene([effect], duration=duration, clear=False)

def main_story_demo(screen: Screen) -> None:
    scenes = [_story_scene(screen)]
    screen.play(scenes, stop_on_resize=True)

if __name__ == "__main__":
    try:
        Screen.wrapper(main_story_demo)
    except ResizeScreenError:
        pass
    sys.exit(0)

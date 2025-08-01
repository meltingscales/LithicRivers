"""
LithicRivers - Main entry point
Copyright (c) 2024 Henry Post. All rights reserved.
"""

import logging
import os.path
import sys

from asciimatics.exceptions import ResizeScreenError
from asciimatics.screen import Screen

from lithicrivers.game import Game
from lithicrivers.model.modelpleasemoveme import StopGameError
from lithicrivers.settings import GAME_NAME, LOGFILENAME, LOGGINGLEVEL
from lithicrivers.ui import demo

# Add custom log levels for verbose debugging
logging.addLevelName(5, "SILLY")
logging.addLevelName(1, "INSANE")

if os.path.exists(LOGFILENAME):
    os.remove(LOGFILENAME)

logging.basicConfig(filename=LOGFILENAME, level=LOGGINGLEVEL)

if __name__ == "__main__":
    # Check for debug flag
    if "--debug" in sys.argv:
        try:
            import pydevd
            print("🐛 Remote debugging enabled!")
            print("📝 In PyCharm: Run -> Attach to Process -> Select this Python process")
            print("🔗 Or use: Run -> Edit Configurations -> + -> Python Debug Server")
            print("🌐 Debug server will be available on localhost:5678")
            # Don't suspend on startup, only when breakpoints are hit
            pydevd.settrace(suspend=False, trace_only_current_thread=True)
        except ImportError:
            print("⚠️  pydevd not installed. Install with: pip install pydevd")
            print("   Or use: uv add pydevd")
    
    GAME = Game()

    print(f"Welcome to {GAME_NAME}.\nSee '{LOGFILENAME}' for logs.")

    last_scene = None
    while GAME.running:
        try:
            logging.debug("Running Screen.wrapper()")
            Screen.wrapper(demo, catch_interrupt=True, arguments=[last_scene, GAME])
        except StopGameError:
            logging.debug("Caught StopGameError!")
            GAME.running = False
        except ResizeScreenError:
            logging.debug("Caught ResizeScreenError !")
            pass
            # Screen rendering stops and re-starts if we get a ResizeScreenError since we're in a while loop...

    print("Game is no longer running :3c")
    print("Goodbye!")

    exit(0)

# See PyCharm help at https://www.jetbrains.com/help/pycharm/

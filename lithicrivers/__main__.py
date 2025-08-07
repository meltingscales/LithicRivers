"""
LithicRivers - Main entry point
Copyright (c) 2024 Henry Post. All rights reserved.
"""

from pathlib import Path
import sys
import time
from asciimatics.exceptions import ResizeScreenError
from asciimatics.screen import Screen

from lithicrivers.game.game import Game
from lithicrivers.game.game_save_manager import GameSaveManager
from lithicrivers.logging_config import get_logger, setup_logging
from lithicrivers.model.model import StopGameError
from lithicrivers.settings import DEFAULT_SEED, GAME_NAME, LOGFILENAME
from lithicrivers.ui import demo

# Setup logging
setup_logging()
logger = get_logger(__name__)

if __name__ == "__main__":
    # Check for debug flag
    if "--debug" in sys.argv:
        try:
            import pydevd

            print("🐛 Remote debugging enabled!")
            print(
                "📝 In PyCharm: Run -> Attach to Process -> Select this Python process"
            )
            print("🔗 Or use: Run -> Edit Configurations -> + -> Python Debug Server")
            print("🌐 Debug server will be available on localhost:5678")
            # Don't suspend on startup, only when breakpoints are hit
            pydevd.settrace(suspend=False, trace_only_current_thread=True)
        except ImportError:
            print("⚠️  pydevd not installed. Install with: pip install pydevd")
            print("   Or use: uv add pydevd")

    # Initialize save manager
    save_manager = GameSaveManager()
    
    # Try to load existing save, creating a new one if it doesn't exist
    GAME = save_manager.load_game_or_create_new()

    print(f"Welcome to {GAME_NAME}.\nSee '{LOGFILENAME}' for logs.")

    last_scene = None
    while GAME.running:
        try:
            logger.debug("Running Screen.wrapper()")
            Screen.wrapper(demo, catch_interrupt=True, arguments=[last_scene, GAME, save_manager])
            
            # Check for automatic snapshot creation
            if save_manager.should_create_automatic_snapshot(GAME.gametick):
                logger.info(f"Creating automatic snapshot at tick {GAME.gametick}")
                save_manager.create_automatic_snapshot_if_needed(GAME)
                # Clean up old snapshots to prevent disk space issues
                save_manager.cleanup_old_snapshots(max_snapshots=10)
                
        except StopGameError:
            logger.debug("Caught StopGameError!")
            break
        except ResizeScreenError:
            logger.debug("Caught ResizeScreenError !")
            pass
            # Screen rendering stops and re-starts if we get a ResizeScreenError since we're in a while loop...

    print("Game is no longer running :3c")
    print("Goodbye! Please wait for worldgen and thread pool to shut down.")

    for i in range(5):
        print("! " * 20)
        print("DO NOT EXIT THE GAME OR PRESS CTRL-C! YOU WILL LOSE GAME PROGRESS IF YOU DO!")
    time.sleep(2)

    # Save the game using the save manager
    print("Saving game...")
    success = save_manager.save_game(GAME)
    if success:
        print(f"✅ Game saved successfully to {save_manager.saves_folder}")
        
        # Create a snapshot for backup
        print("Creating backup snapshot...")
        snapshot_success = save_manager.create_snapshot(GAME)
        if snapshot_success:
            print(f"✅ Backup snapshot created in {save_manager.snapshots_folder}")
        else:
            print("⚠️  Failed to create backup snapshot")
    else:
        print("❌ Failed to save game!")


    exit(0)

# See PyCharm help at https://www.jetbrains.com/help/pycharm/

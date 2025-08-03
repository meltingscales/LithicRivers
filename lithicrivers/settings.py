import logging
import os
import multiprocessing

from lithicrivers.config_manager import config_manager
from lithicrivers.model.model import Viewport
from lithicrivers.keymap import Keymap

# Load settings from config manager
GAME_NAME = config_manager.get_setting("game", "GAME_NAME")
LOGFILENAME = config_manager.get_setting("game", "LOGFILENAME")
DEVELOPER_MODE = config_manager.get_setting("game", "DEVELOPER_MODE")
DEFAULT_SEED = config_manager.get_setting("game", "DEFAULT_SEED")

# Load logging level with support for custom levels
log_level_str = config_manager.get_setting("game", "LOGGINGLEVEL")
if log_level_str.isdigit():
    LOGGINGLEVEL = int(log_level_str)
else:
    LOGGINGLEVEL = getattr(logging, log_level_str)


# Load world settings based on environment
if os.environ.get("TESTING") == "1":
    DEFAULT_PLAYER_POSITION = config_manager.get_vector_setting(
        "world", "DEFAULT_PLAYER_POSITION", "testing"
    )
else:
    DEFAULT_PLAYER_POSITION = config_manager.get_vector_setting(
        "world", "DEFAULT_PLAYER_POSITION", "production"
    )

# Load viewport settings
VIEWPORT_RADIUS = config_manager.get_vector_setting("viewport", "VIEWPORT_RADIUS")
VIEWPORT_WIGGLE = config_manager.get_setting("viewport", "VIEWPORT_WIGGLE")

# Create default viewport
DEFAULT_VIEWPORT = Viewport.generate_centered(
    DEFAULT_PLAYER_POSITION, radius=VIEWPORT_RADIUS
)

# CPU thread detection and configuration
def get_max_cpu_threads():
    """Get the maximum number of CPU threads to use."""
    # Try environment variable first
    env_threads = os.environ.get("MAX_CPU_THREADS")
    if env_threads:
        try:
            return int(env_threads)
        except ValueError:
            pass
    
    # Try to get from config
    try:
        config_threads = config_manager.get_setting("performance", "MAX_CPU_THREADS")
        if config_threads:
            return int(config_threads)
    except:
        pass
    
    # Auto-detect CPU count, default to 64 if detection fails
    try:
        cpu_count = multiprocessing.cpu_count()
        # Use 75% of available cores to avoid overwhelming the system
        return max(1, min(cpu_count, int(cpu_count * 0.75)))
    except:
        return 64

MAX_CPU_THREADS = get_max_cpu_threads()

# Create global keymap instance
KEYMAP = Keymap()

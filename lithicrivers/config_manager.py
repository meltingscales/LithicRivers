import json
import logging
import sys
from pathlib import Path
from typing import Any, List, Union

from lithicrivers.model.vector import VectorN


class ConfigManager:
    """Manages loading and saving of configuration files (keybinds and settings)."""

    def __init__(self):
        self.config_dir = self._get_config_directory()
        self.keybinds_file = self.config_dir / "keybinds.json"
        self.settings_file = self.config_dir / "settings.json"

        # Load configurations
        self.keybinds = self._load_keybinds()
        self.settings = self._load_settings()

    def _get_config_directory(self) -> Path:
        """Get the configuration directory, creating it if it doesn't exist."""
        if getattr(sys, "frozen", False):
            # Running as compiled executable
            config_dir = Path(sys.executable).parent / "config"
        else:
            # Running as Python script
            config_dir = Path(__file__).parent.parent / "config"

        config_dir.mkdir(exist_ok=True)
        return config_dir

    def _load_keybinds(self) -> dict[str, Any]:
        """Load keybinds from JSON file or create default if file doesn't exist."""
        default_keybinds = {
            "movement": {
                "MOVE_NORTHWEST": "7",
                "MOVE_NORTH": "8",
                "MOVE_NORTHEAST": "9",
                "MOVE_WEST": "4",
                "WAIT": "5",
                "MOVE_EAST": "6",
                "MOVE_SOUTHWEST": "1",
                "MOVE_SOUTH": "2",
                "MOVE_SOUTHEAST": "3",
                "MOVE_UP": "q",
                "MOVE_DOWN": "e"
            },
            "viewport": {
                "RESET_VIEWPORT": "r",
                "SLIDE_VIEWPORT_WEST": "[",
                "SLIDE_VIEWPORT_EAST": "]",
                "TOGGLE_VIEWPORT": "v"
            },
            "scale": {
                "SCALE_UP": ["=", "+"],
                "SCALE_DOWN": "-"
            },
            "action": {
                "MINE": "u",
                "INTERACT": "i"
            }
        }

        return self._load_json_file(self.keybinds_file, default_keybinds)

    def _load_settings(self) -> dict[str, Any]:
        """Load settings from JSON file or create default if file doesn't exist."""
        default_settings = {
            "game": {
                "GAME_NAME": "LithicRivers",
                "LOGFILENAME": "LithicRivers.log",
                "LOGGINGLEVEL": "INFO",
            },
            "world": {
                "DEFAULT_SIZE_RADIUS": {
                    "production": [50, 50, 3],
                    "testing": [5, 5, 1],
                },
                "DEFAULT_PLAYER_POSITION": {
                    "production": [25, 25, 0],
                    "testing": [0, 0, 0],
                },
            },
            "viewport": {"VIEWPORT_RADIUS": [8, 8, 0], "VIEWPORT_WIGGLE": 2},
        }

        return self._load_json_file(self.settings_file, default_settings)

    def _load_json_file(
        self, file_path: Path, default_data: dict[str, Any]
    ) -> dict[str, Any]:
        """Load JSON file or create default if file doesn't exist."""
        try:
            if file_path.exists():
                with open(file_path) as f:
                    return json.load(f)
            else:
                # Create default file
                with open(file_path, "w") as f:
                    json.dump(default_data, f, indent=2)
                logging.info(f"Created default config file: {file_path}")
                return default_data
        except (OSError, json.JSONDecodeError) as e:
            logging.warning(
                f"Error loading config file {file_path}: {e}. Using defaults."
            )
            return default_data

    def get_keybind(self, category: str, key_name: str) -> List[str]:
        """Get a keybind value."""
        return_value = self.keybinds.get(category, {}).get(key_name, [])
        if isinstance(return_value, str):
            return [return_value]
        return return_value

    def get_setting(self, category: str, key: str) -> Any:
        """Get a setting value."""
        return self.settings.get(category, {}).get(key)

    def get_vector_setting(
        self, category: str, key: str, environment: str = "production"
    ) -> VectorN:
        """Get a vector setting, handling environment-specific values."""
        value = self.settings.get(category, {}).get(key, {})
        coords = value.get(environment, [0, 0, 0]) if isinstance(value, dict) else value

        return VectorN(*coords)

    def save_keybinds(self):
        """Save current keybinds to file."""
        try:
            with open(self.keybinds_file, "w") as f:
                json.dump(self.keybinds, f, indent=2)
        except OSError as e:
            logging.error(f"Error saving keybinds: {e}")

    def save_settings(self):
        """Save current settings to file."""
        try:
            with open(self.settings_file, "w") as f:
                json.dump(self.settings, f, indent=2)
        except OSError as e:
            logging.error(f"Error saving settings: {e}")

    def update_keybind(self, category: str, key_name: str, value: List[str]):
        """Update a keybind value."""
        if category not in self.keybinds:
            self.keybinds[category] = {}
        self.keybinds[category][key_name] = value
        self.save_keybinds()

    def update_setting(self, category: str, key: str, value: Any):
        """Update a setting value."""
        if category not in self.settings:
            self.settings[category] = {}
        self.settings[category][key] = value
        self.save_settings()


# Global config manager instance
config_manager = ConfigManager()

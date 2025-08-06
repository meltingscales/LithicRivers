import json
import logging
import sys
from pathlib import Path
from typing import Any

from lithicrivers.model.vector import VectorN


class ConfigManager:
    """Manages loading and saving of configuration files (keybinds and settings)."""

    def __init__(self) -> None:
        self.config_dir = self._get_config_directory()
        self.config_file = self.config_dir / "lithicrivers-config.json"

        # Load configurations
        self.config = self._load_config()
        self.keybinds = self.config.get("keybinds", {})
        self.settings = self.config.get("settings", {})

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

    def _load_config(self) -> dict[str, Any]:
        """Load configuration from JSON file or create default if file doesn't exist."""
        default_config = {
            "keybinds": {
                "movement": {
                    "MOVE_NORTHWEST": ["NUMPAD_7"],
                    "MOVE_NORTH": ["NUMPAD_8"],
                    "MOVE_NORTHEAST": ["NUMPAD_9"],
                    "MOVE_WEST": ["NUMPAD_4"],
                    "WAIT": ["NUMPAD_5"],
                    "MOVE_EAST": ["NUMPAD_6"],
                    "MOVE_SOUTHWEST": ["NUMPAD_1"],
                    "MOVE_SOUTH": ["NUMPAD_2"],
                    "MOVE_SOUTHEAST": ["NUMPAD_3"],
                    "MOVE_UP": ["q"],
                    "MOVE_DOWN": ["e"],
                },
                "viewport": {
                    "RESET_VIEWPORT": ["r"],
                    "SLIDE_VIEWPORT_WEST": ["["],
                    "SLIDE_VIEWPORT_EAST": ["]"],
                    "TOGGLE_VIEWPORT": ["v"],
                },
                "scale": {"SCALE_UP": ["=", "+"], "SCALE_DOWN": ["-"]},
                "action": {"MINE": ["u"], "INTERACT": ["i"], "PICKUP_ITEMS": ["g"]},
                "ui": {"CLOSE_HELP_MENU": ["ESCAPE"], "OPEN_COMMAND_MENU": ["/"]},
                "inventory": {
                    "DROP_ITEM": ["d"],
                    "DESTROY_ITEM": ["x"],
                    "CHEAT_DUPLICATE_ITEM": ["."],
                },
            },
            "settings": {
                "game": {
                    "GAME_NAME": "LithicRivers",
                    "LOGFILENAME": "LithicRivers.log",
                    "LOGGINGLEVEL": "INFO",
                    "DEVELOPER_MODE": True,
                    "DEFAULT_SEED": 4669201609,
                    "DEFAULT_PLAYER_NAME": "Inigo Montoya",
                },
                "world": {
                    "DEFAULT_SIZE_RADIUS": {
                        "production": [50, 50, 3],
                        "testing": [3, 3, 1],
                    },
                    "DEFAULT_PLAYER_POSITION": {
                        "production": [25, 25, 0],
                        "testing": [0, 0, 0],
                    },
                },
                "viewport": {"VIEWPORT_RADIUS": [8, 8, 0], "VIEWPORT_WIGGLE": 2},
                "performance": {"MAX_CPU_THREADS": 64},
                "worldgen": {"CHUNK_SIZE": 8},
            },
        }

        return self._load_json_file(self.config_file, default_config)

    def _load_json_file(
        self, file_path: Path, default_data: dict[str, Any]
    ) -> dict[str, Any]:
        """Load JSON file or create default if file doesn't exist."""
        try:
            if file_path.exists():
                with open(file_path) as f:
                    data = json.load(f)
                    if isinstance(data, dict):
                        return data
                    else:
                        logging.warning(f"Invalid config file format: {file_path}")
                        return default_data
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

    def get_keybind(self, category: str, key_name: str) -> frozenset[str]:
        """Get a keybind value."""
        return_value = self.keybinds.get(category, {}).get(key_name, frozenset())

        # convert to frozenset as it may be a list of strings
        return frozenset(return_value)

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

    def save_config(self) -> None:
        """Save current configuration to file."""
        try:
            # Update the config dict with current keybinds and settings
            self.config["keybinds"] = self.keybinds
            self.config["settings"] = self.settings
            
            with open(self.config_file, "w") as f:
                json.dump(self.config, f, indent=2)
        except OSError as e:
            logging.error(f"Error saving config: {e}")

    def save_keybinds(self) -> None:
        """Save current keybinds to file."""
        self.save_config()

    def save_settings(self) -> None:
        """Save current settings to file."""
        self.save_config()

    def update_keybind(self, category: str, key_name: str, value: list[str]) -> None:
        """Update a keybind value."""
        if category not in self.keybinds:
            self.keybinds[category] = {}
        self.keybinds[category][key_name] = value
        self.save_keybinds()

    def update_setting(self, category: str, key: str, value: Any) -> None:
        """Update a setting value."""
        if category not in self.settings:
            self.settings[category] = {}
        self.settings[category][key] = value
        self.save_settings()


# Global config manager instance
config_manager = ConfigManager()

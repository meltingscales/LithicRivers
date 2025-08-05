"""
Test the merged configuration system.
Copyright (c) 2024 Henry Post. All rights reserved.
"""

import unittest
import tempfile
import json
import os
from pathlib import Path

from lithicrivers.config_manager import ConfigManager


class TestConfigMerge(unittest.TestCase):
    """Test the merged configuration system."""

    def setUp(self):
        """Set up test fixtures."""
        # Create a temporary directory for testing
        self.temp_dir = tempfile.mkdtemp()
        self.config_dir = Path(self.temp_dir) / "config"
        self.config_dir.mkdir(exist_ok=True)
        
        # Create a test config file
        self.test_config_file = self.config_dir / "lithicrivers-config.json"
        
        # Store original config directory method
        self.original_get_config_directory = ConfigManager._get_config_directory

    def tearDown(self):
        """Clean up test fixtures."""
        # Restore original method
        ConfigManager._get_config_directory = self.original_get_config_directory
        
        # Clean up temporary directory
        import shutil
        shutil.rmtree(self.temp_dir)

    def test_merged_config_structure(self):
        """Test that the merged config has the correct structure."""
        # Mock the config directory to use our test directory
        test_config_dir = self.config_dir
        def mock_get_config_directory(self):
            return test_config_dir
        
        ConfigManager._get_config_directory = mock_get_config_directory
        
        # Create a config manager
        config_manager = ConfigManager()
        
        # Check that the config has both keybinds and settings
        self.assertIn("keybinds", config_manager.config)
        self.assertIn("settings", config_manager.config)
        
        # Check that keybinds and settings are accessible
        self.assertIsInstance(config_manager.keybinds, dict)
        self.assertIsInstance(config_manager.settings, dict)
        
        # Check that the pickup items keybind is present
        pickup_keybind = config_manager.get_keybind("action", "PICKUP_ITEMS")
        self.assertEqual(pickup_keybind, frozenset(["g"]))

    def test_config_save_and_load(self):
        """Test that the config can be saved and loaded correctly."""
        # Mock the config directory to use our test directory
        test_config_dir = self.config_dir
        def mock_get_config_directory(self):
            return test_config_dir
        
        ConfigManager._get_config_directory = mock_get_config_directory
        
        # Create a config manager
        config_manager = ConfigManager()
        
        # Modify a keybind
        config_manager.update_keybind("action", "PICKUP_ITEMS", ["h"])
        
        # Modify a setting
        config_manager.update_setting("game", "DEFAULT_PLAYER_NAME", "Test Player")
        
        # Verify the changes are in memory
        self.assertEqual(config_manager.get_keybind("action", "PICKUP_ITEMS"), frozenset(["h"]))
        self.assertEqual(config_manager.get_setting("game", "DEFAULT_PLAYER_NAME"), "Test Player")
        
        # Verify the config file was created
        self.assertTrue(self.test_config_file.exists())
        
        # Load the config file and verify it contains the changes
        with open(self.test_config_file) as f:
            saved_config = json.load(f)
        
        self.assertEqual(saved_config["keybinds"]["action"]["PICKUP_ITEMS"], ["h"])
        self.assertEqual(saved_config["settings"]["game"]["DEFAULT_PLAYER_NAME"], "Test Player")

    def test_backward_compatibility(self):
        """Test that the existing API still works."""
        # Mock the config directory to use our test directory
        test_config_dir = self.config_dir
        def mock_get_config_directory(self):
            return test_config_dir
        
        ConfigManager._get_config_directory = mock_get_config_directory
        
        # Create a config manager
        config_manager = ConfigManager()
        
        # Test that existing methods still work
        self.assertEqual(config_manager.get_keybind("action", "MINE"), frozenset(["u"]))
        self.assertEqual(config_manager.get_setting("game", "GAME_NAME"), "LithicRivers")
        
        # Test vector settings
        viewport_radius = config_manager.get_vector_setting("viewport", "VIEWPORT_RADIUS")
        self.assertEqual(viewport_radius.x, 8)
        self.assertEqual(viewport_radius.y, 8)
        self.assertEqual(viewport_radius.z, 0)


if __name__ == "__main__":
    unittest.main() 
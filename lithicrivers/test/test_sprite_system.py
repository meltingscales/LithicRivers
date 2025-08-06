"""
Unit tests for the sprite system.
Tests sprite loading, rendering, and scale handling.
"""

import unittest
import tempfile
import json
from pathlib import Path

from lithicrivers.sprite_loader import SpriteData, SpriteLoader, get_sprite_loader
from lithicrivers.game import Fluid, Entities, VectorN, Game
from lithicrivers.test.test_fixtures import OptimizedTestCase


class TestSpriteSystem(OptimizedTestCase):
    """Test the sprite system."""

    def setUp(self):
        """Set up test fixtures."""
        self.game = self.get_game(seed=42)
        self.sprites = ["~", "~~\n~~", "~~~\n~~~\n~~~"]
        self.sprite_data = SpriteData("water", "blue", "Flowing water", self.sprites)
    
    def test_sprite_data_creation(self):
        """Test SpriteData creation."""
        self.assertEqual(self.sprite_data.name, "water")
        self.assertEqual(self.sprite_data.color, "blue")
        self.assertEqual(self.sprite_data.description, "Flowing water")
        self.assertEqual(self.sprite_data.sprites, self.sprites)
    
    def test_get_sprite_valid_scales(self):
        """Test getting sprites at valid scales."""
        self.assertEqual(self.sprite_data.get_sprite(1), "~")
        self.assertEqual(self.sprite_data.get_sprite(2), "~~\n~~")
        self.assertEqual(self.sprite_data.get_sprite(3), "~~~\n~~~\n~~~")
    
    def test_get_sprite_invalid_scales(self):
        """Test getting sprites at invalid scales."""
        # Scale 0 (invalid)
        self.assertEqual(self.sprite_data.get_sprite(0), "?")
        # Scale 4 (out of bounds)
        self.assertEqual(self.sprite_data.get_sprite(4), "?")
        # Negative scale
        self.assertEqual(self.sprite_data.get_sprite(-1), "?")


class TestSpriteLoader(unittest.TestCase):
    """Test the SpriteLoader class."""
    
    def setUp(self):
        """Set up test environment."""
        self.temp_dir = tempfile.mkdtemp()
        self.sprite_loader = SpriteLoader(Path(self.temp_dir))
        
        # Create test sprite structure
        self.test_sprite_dir = Path(self.temp_dir) / "fluids" / "test.lrsprite"
        self.test_sprite_dir.mkdir(parents=True, exist_ok=True)
        
        # Create test data.json
        test_data = {
            "name": "test",
            "color": "green",
            "description": "Test sprite",
            "scales": [1, 2, 3]
        }
        with open(self.test_sprite_dir / "data.json", 'w') as f:
            json.dump(test_data, f)
        
        # Create test sprites.txt
        test_sprites = "~\n~~\n~~\n~~~\n~~~\n~~~"
        with open(self.test_sprite_dir / "sprites.txt", 'w') as f:
            f.write(test_sprites)
    
    def tearDown(self):
        """Clean up test environment."""
        import shutil
        shutil.rmtree(self.temp_dir)
    
    def test_load_sprite_success(self):
        """Test successful sprite loading."""
        sprite_data = self.sprite_loader.load_sprite("test", "fluids")
        
        self.assertIsNotNone(sprite_data)
        self.assertEqual(sprite_data.name, "test")
        self.assertEqual(sprite_data.color, "green")
        self.assertEqual(sprite_data.description, "Test sprite")
        self.assertEqual(sprite_data.sprites, ["~", "~~\n~~", "~~~\n~~~\n~~~"])
    
    def test_load_sprite_not_found(self):
        """Test loading non-existent sprite."""
        with self.assertRaises(ValueError) as context:
            self.sprite_loader.load_sprite("nonexistent", "fluids")
        
        # Check that the error message includes available sprites
        error_message = str(context.exception)
        self.assertIn("nonexistent", error_message)
        self.assertIn("fluids", error_message)
        self.assertIn("Available sprites", error_message)
    
    def test_load_sprite_missing_data_json(self):
        """Test loading sprite with missing data.json."""
        # Create a sprite directory without data.json
        sprite_dir = Path(self.temp_dir) / "fluids" / "missing_data.lrsprite"
        sprite_dir.mkdir(parents=True, exist_ok=True)
        
        # Create sprites.txt but no data.json
        with open(sprite_dir / "sprites.txt", "w") as f:
            f.write("~\n~~\n~~")
        
        with self.assertRaises(ValueError) as context:
            self.sprite_loader.load_sprite("missing_data", "fluids")
        
        error_message = str(context.exception)
        self.assertIn("Missing data.json", error_message)
    
    def test_load_sprite_missing_sprites_txt(self):
        """Test loading sprite with missing sprites.txt."""
        # Create a sprite directory without sprites.txt
        sprite_dir = Path(self.temp_dir) / "fluids" / "missing_sprites.lrsprite"
        sprite_dir.mkdir(parents=True, exist_ok=True)
        
        # Create data.json but no sprites.txt
        with open(sprite_dir / "data.json", "w") as f:
            f.write('{"name": "missing_sprites", "color": "blue", "description": "test"}')
        
        with self.assertRaises(ValueError) as context:
            self.sprite_loader.load_sprite("missing_sprites", "fluids")
        
        error_message = str(context.exception)
        self.assertIn("Missing sprites.txt", error_message)
    
    def test_sprite_caching(self):
        """Test that sprites are cached."""
        # Load sprite first time
        sprite_data1 = self.sprite_loader.load_sprite("test", "fluids")
        self.assertIsNotNone(sprite_data1)
        
        # Load sprite second time (should be cached)
        sprite_data2 = self.sprite_loader.load_sprite("test", "fluids")
        self.assertIsNotNone(sprite_data2)
        
        # Should be the same object (cached)
        self.assertIs(sprite_data1, sprite_data2)
    
    def test_get_available_sprites(self):
        """Test getting available sprites."""
        sprites = self.sprite_loader.get_available_sprites("fluids")
        self.assertIn("test", sprites)
    
    def test_clear_cache(self):
        """Test clearing the sprite cache."""
        # Load a sprite
        sprite_data1 = self.sprite_loader.load_sprite("test", "fluids")
        self.assertIsNotNone(sprite_data1)
        
        # Clear cache
        self.sprite_loader.clear_cache()
        
        # Load again (should be new object)
        sprite_data2 = self.sprite_loader.load_sprite("test", "fluids")
        self.assertIsNotNone(sprite_data2)
        
        # Should be different objects (not cached)
        self.assertIsNot(sprite_data1, sprite_data2)


class TestFluidSpriteRendering(unittest.TestCase):
    """Test fluid sprite rendering at different scales."""
    
    def setUp(self):
        """Set up test environment."""
        self.game = Game(seed=42)
    
    def test_water_sprite_scales(self):
        """Test water sprite rendering at all scales."""
        water = Entities.water(VectorN(0, 0, 0))
        
        # Test scale 1 (1x1)
        sprite_1 = water.render_sprite(1)
        self.assertEqual(sprite_1, "~")
        
        # Test scale 2 (2x2)
        sprite_2 = water.render_sprite(2)
        self.assertEqual(sprite_2, "~~\n~~")
        
        # Test scale 3 (3x3)
        sprite_3 = water.render_sprite(3)
        self.assertEqual(sprite_3, "~~~\n~~~\n~~~")
    
    def test_lava_sprite_scales(self):
        """Test lava sprite rendering at all scales."""
        lava = Entities.lava(VectorN(0, 0, 0))
        
        # Test scale 1 (1x1)
        sprite_1 = lava.render_sprite(1)
        self.assertEqual(sprite_1, "=")
        
        # Test scale 2 (2x2)
        sprite_2 = lava.render_sprite(2)
        self.assertEqual(sprite_2, "==\n==")
        
        # Test scale 3 (3x3)
        sprite_3 = lava.render_sprite(3)
        self.assertEqual(sprite_3, "===\n===\n===")
    
    def test_acid_sprite_scales(self):
        """Test acid sprite rendering at all scales."""
        acid = Entities.acid(VectorN(0, 0, 0))
        
        # Test scale 1 (1x1)
        sprite_1 = acid.render_sprite(1)
        self.assertEqual(sprite_1, "*")
        
        # Test scale 2 (2x2)
        sprite_2 = acid.render_sprite(2)
        self.assertEqual(sprite_2, "**\n**")
        
        # Test scale 3 (3x3)
        sprite_3 = acid.render_sprite(3)
        self.assertEqual(sprite_3, "***\n***\n***")
    
    def test_oil_sprite_scales(self):
        """Test oil sprite rendering at all scales."""
        oil = Entities.oil(VectorN(0, 0, 0))
        
        # Test scale 1 (1x1)
        sprite_1 = oil.render_sprite(1)
        self.assertEqual(sprite_1, "o")
        
        # Test scale 2 (2x2)
        sprite_2 = oil.render_sprite(2)
        self.assertEqual(sprite_2, "oo\noo")
        
        # Test scale 3 (3x3)
        sprite_3 = oil.render_sprite(3)
        self.assertEqual(sprite_3, "ooo\nooo\nooo")
    
    def test_blood_sprite_scales(self):
        """Test blood sprite rendering at all scales."""
        blood = Entities.blood(VectorN(0, 0, 0))
        
        # Test scale 1 (1x1)
        sprite_1 = blood.render_sprite(1)
        self.assertEqual(sprite_1, "%")
        
        # Test scale 2 (2x2)
        sprite_2 = blood.render_sprite(2)
        self.assertEqual(sprite_2, "%%\n%%")
        
        # Test scale 3 (3x3)
        sprite_3 = blood.render_sprite(3)
        self.assertEqual(sprite_3, "%%%\n%%%\n%%%")
    
    def test_invalid_scales(self):
        """Test rendering at invalid scales."""
        water = Entities.water(VectorN(0, 0, 0))
        
        # Test scale 0 (should throw exception)
        with self.assertRaises(ValueError):
            water.render_sprite(0)
        
        # Test scale 4 (should throw exception)
        with self.assertRaises(ValueError):
            water.render_sprite(4)
    
    def test_fluid_colors(self):
        """Test that fluids have correct colors."""
        fluids = [
            (Entities.water(VectorN(0, 0, 0)), "blue"),
            (Entities.lava(VectorN(0, 0, 0)), "red"),
            (Entities.acid(VectorN(0, 0, 0)), "green"),
            (Entities.oil(VectorN(0, 0, 0)), "yellow"),
            (Entities.blood(VectorN(0, 0, 0)), "red"),
        ]
        
        for fluid, expected_color in fluids:
            color = fluid.get_color()
            self.assertEqual(color, expected_color)


class TestSpriteSystemIntegration(unittest.TestCase):
    """Test integration between sprite loader and fluid rendering."""
    
    def test_external_sprite_loading(self):
        """Test that fluids load sprites from external files."""
        # Get the sprite loader
        sprite_loader = get_sprite_loader()
        
        # Check that external sprites are available
        available_sprites = sprite_loader.get_available_sprites("fluids")
        expected_sprites = ["water", "lava", "acid", "oil", "blood"]
        
        for sprite in expected_sprites:
            self.assertIn(sprite, available_sprites)
        
        # Test that each sprite can be loaded
        for sprite_name in expected_sprites:
            sprite_data = sprite_loader.load_sprite(sprite_name, "fluids")
            self.assertIsNotNone(sprite_data)
            self.assertEqual(sprite_data.name, sprite_name)
    
    def test_fallback_to_hardcoded(self):
        """Test that non-existent sprites throw exceptions."""
        # Create a fluid with a non-existent sprite type
        # This should throw an exception since we no longer have fallbacks
        with self.assertRaises(ValueError):
            Fluid("nonexistent", VectorN(0, 0, 0))


def run_sprite_system_tests():
    """Run all sprite system tests."""
    print("🧪 Running Sprite System Tests")
    print("=" * 40)
    
    # Create test suite
    test_suite = unittest.TestSuite()
    
    # Add test classes
    test_suite.addTest(unittest.makeSuite(TestSpriteData))
    test_suite.addTest(unittest.makeSuite(TestSpriteLoader))
    test_suite.addTest(unittest.makeSuite(TestFluidSpriteRendering))
    test_suite.addTest(unittest.makeSuite(TestSpriteSystemIntegration))
    
    # Run tests
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(test_suite)
    
    # Print summary
    print(f"\n📊 Test Summary:")
    print(f"  Tests run: {result.testsRun}")
    print(f"  Failures: {len(result.failures)}")
    print(f"  Errors: {len(result.errors)}")
    
    if result.failures:
        print(f"\n❌ Failures:")
        for test, traceback in result.failures:
            print(f"  {test}: {traceback}")
    
    if result.errors:
        print(f"\n❌ Errors:")
        for test, traceback in result.errors:
            print(f"  {test}: {traceback}")
    
    if result.wasSuccessful():
        print(f"\n✅ All tests passed!")
    else:
        print(f"\n❌ Some tests failed!")
    
    return result.wasSuccessful()


if __name__ == "__main__":
    run_sprite_system_tests() 
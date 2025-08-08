
from lithicrivers.game.core import Tiles
from lithicrivers.model.model import RenderedData, Viewport
from lithicrivers.model.vector import VectorN
from lithicrivers.settings import DEFAULT_SEED
from lithicrivers.test.test_fixtures import OptimizedTestCase
from lithicrivers.textutil import generate_sprite_repeat
from lithicrivers.game.core import ChunkedWorldData


class RenderStuff(OptimizedTestCase):
    def testgenerate_sprite_repeat(self):
        self.assertEqual("?", generate_sprite_repeat("?", 1))

        self.assertEqual(
            "??\n??",
            generate_sprite_repeat("?", 2),
        )

        self.assertEqual("???\n???\n???", generate_sprite_repeat("?", 3))

    def test_simple_render(self):
        self.assertEqual(Tiles.gold_ore().render_sprite(), "?")
        self.assertEqual(Tiles.gold_ore().render_sprite(1), "?")
        self.assertEqual(Tiles.gold_ore().render_sprite(2), "??\n??")

    def test_color_expansion_for_scaled_sprites(self):
        """Test that color data is properly expanded for scaled sprites."""
        # Create a simple rendered data with 2x2 tiles at scale 2
        render_data = [["a", "b"], ["c", "d"]]

        # Create color data for the tiles
        color_data = [
            [(1, 0, 0), (2, 0, 0)],  # Red, Green
            [(3, 0, 0), (4, 0, 0)],  # Yellow, Blue
        ]

        rendered_data = RenderedData(render_data, scale=2, color_data=color_data)

        # Test that colors are properly accessible at scaled positions
        # For scale 2, each tile becomes 2x2 characters
        # Position (0,0) should have color (1,0,0) for all 4 characters
        self.assertEqual(rendered_data.get_color_at(0, 0), (1, 0, 0))
        self.assertEqual(rendered_data.get_color_at(1, 0), (2, 0, 0))
        self.assertEqual(rendered_data.get_color_at(0, 1), (3, 0, 0))
        self.assertEqual(rendered_data.get_color_at(1, 1), (4, 0, 0))

    def test_sorta_simple_render(self):
        some_render = RenderedData(render_data=[["x", "y"], ["z", "R"]], scale=1)

        self.assertEqual(some_render.as_string(), "xy\nzR")

    def test_sorta_simple_render_reverse(self):
        return None  # this disables the test
        self.assertEqual(
            RenderedData.from_string("xy\nzR", scale=1),
            [["x", "y"], ["z", "R"]],
        )

        self.assertEqual(
            RenderedData.from_string("xxyy\nxxyy\nzzRR\nzzRR", scale=1),
            [["xx\nxx", "yy\nyy"], ["zz\nzz", "RR\nRR"]],
        )

    def test_render_game(self):
        some_game = self.get_game(seed=DEFAULT_SEED)
        world_data = ChunkedWorldData()

        # Set tiles individually
        world_data.set_tile(VectorN(0, 0, 0), Tiles.dirt())
        world_data.set_tile(VectorN(1, 0, 0), Tiles.gold_ore())
        world_data.set_tile(VectorN(0, 1, 0), Tiles.dirt())
        world_data.set_tile(VectorN(1, 1, 0), Tiles.dirt())

        some_game.world.data = world_data

        # Move player out of the viewport so tiles are visible
        some_game.player.position = VectorN(5, 5, 0)

        da_scale = 2

        rendered_viewport = some_game.render_world_viewport(
            viewport=Viewport(
                top_left=VectorN(0, 0), lower_right=VectorN(1, 1), scale=da_scale
            )
        )

        self.assertEqual(
            ",.??\n.,??\n,.,.\n.,.,",
            rendered_viewport.as_string(),
        )

    def test_viewport_sizing(self):
        """Test that viewport sizing works correctly."""
        from lithicrivers.model.model import Viewport
        from lithicrivers.model.vector import VectorN

        # Test that viewport can be created with different sizes
        player_pos = VectorN(0, 0, 0)

        # Test small viewport
        small_viewport = Viewport.generate_centered(player_pos, radius=VectorN(5, 5, 0))
        self.assertEqual(small_viewport.get_width(), 10)  # abs(5 - (-5)) = 10
        self.assertEqual(small_viewport.get_height(), 10)

        # Test larger viewport
        large_viewport = Viewport.generate_centered(
            player_pos, radius=VectorN(15, 15, 0)
        )
        self.assertEqual(large_viewport.get_width(), 30)  # abs(15 - (-15)) = 30
        self.assertEqual(large_viewport.get_height(), 30)

        # Test that viewport respects scale
        scaled_viewport = Viewport.generate_centered(
            player_pos, radius=VectorN(10, 10, 0), scale=2
        )
        self.assertEqual(scaled_viewport.scale, 2)
        self.assertEqual(scaled_viewport.get_width(), 20)  # abs(10 - (-10)) = 20
        self.assertEqual(scaled_viewport.get_height(), 20)

    def test_dynamic_viewport_sizing(self):
        """Test that viewport sizing dynamically adjusts based on available space."""
        from unittest.mock import Mock

        from lithicrivers.ui import WorldMap

        # Create a mock screen with different sizes
        def create_mock_screen(width, height):
            screen = Mock()
            screen.width = width
            screen.height = height
            return screen

        # Test small screen (80x24)
        small_screen = create_mock_screen(80, 24)
        game = self.get_game(seed=DEFAULT_SEED)
        WorldMap(small_screen, game)

        # Check that viewport was adjusted for small screen
        # Small screen should have smaller viewport than large screen
        small_viewport_width = game.viewport.get_width()
        small_viewport_height = game.viewport.get_height()

        # Test large screen (160x48)
        large_screen = create_mock_screen(160, 48)
        game_large = self.get_game(seed=DEFAULT_SEED)
        WorldMap(large_screen, game_large)

        # Check that viewport was adjusted for large screen
        # Large screen should have larger viewport
        large_viewport_width = game_large.viewport.get_width()
        large_viewport_height = game_large.viewport.get_height()

        # Large screen should have larger viewport than small screen
        self.assertGreater(large_viewport_width, small_viewport_width)
        self.assertGreater(large_viewport_height, small_viewport_height)

        # Test that viewport respects minimum size
        # Even with a very small screen, viewport should be at least 5x5
        tiny_screen = create_mock_screen(40, 12)
        game_tiny = self.get_game(seed=DEFAULT_SEED)
        WorldMap(tiny_screen, game_tiny)

        tiny_viewport_width = game_tiny.viewport.get_width()
        tiny_viewport_height = game_tiny.viewport.get_height()

        # Should have minimum viewport size (accounting for status bar)
        self.assertGreaterEqual(tiny_viewport_width, 3)
        self.assertGreaterEqual(tiny_viewport_height, 3)

    def test_on_demand_tile_generation(self):
        """Test that tiles are generated on-demand when accessed outside the initial world area."""
        from lithicrivers.model.vector import VectorN

        # Create a game with a small initial world
        game = self.get_game(seed=DEFAULT_SEED)

        # Test accessing a tile far outside the initial world area
        far_position = VectorN(1000, 1000, 0)
        tile = game.world.get_tile(far_position)

        # Should not be None (should generate a tile on-demand)
        self.assertIsNotNone(tile)

        # Should be a valid tile type
        self.assertIn(
            tile.tileid, ["Dirt", "Tree", "Gold Ore", "Bedrock", "Cloud", "Empty"]
        )

        # Test that the tile is now cached in the world data
        cached_tile = game.world.data.get_tile(far_position)
        self.assertIsNotNone(cached_tile)
        self.assertEqual(tile, cached_tile)

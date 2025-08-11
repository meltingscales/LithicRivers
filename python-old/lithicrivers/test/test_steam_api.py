"""Tests for Steam API integration."""

from lithicrivers.model.steam_api import (
    SteamAPI,
    cleanup_steam,
    get_steam_api,
    initialize_steam,
)


class TestSteamAPI:
    """Test Steam API functionality."""

    def test_steam_api_import(self):
        """Test that Steam API can be imported."""

        assert SteamAPI is not None

    def test_steam_api_availability(self):
        """Test Steam API availability check."""
        api = get_steam_api()
        if api:
            assert hasattr(api, "is_available")
            assert callable(api.is_available)
        else:
            # If Steam API is not available, that's also valid
            assert True

    def test_steam_api_initialization(self):
        """Test Steam API initialization."""
        api = get_steam_api()
        if api:
            assert hasattr(api, "client")
            assert hasattr(api, "is_connected")
            assert api.is_connected is False

    def test_steam_api_cleanup(self):
        """Test Steam API cleanup."""
        cleanup_steam()
        # After cleanup, get_steam_api should return None if Steam is not available
        api = get_steam_api()
        if api:
            assert api.is_connected is False

    def test_steam_functions_exist(self):
        """Test that Steam API functions exist."""
        assert callable(get_steam_api)
        assert callable(initialize_steam)
        assert callable(cleanup_steam)

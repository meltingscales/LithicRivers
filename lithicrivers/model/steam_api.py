"""
Steam API integration for LithicRivers.

This module provides integration with Valve's Steam API using the steam[client] package.
"""

try:
    import steam.client
    import steam.guard

    STEAM_AVAILABLE = True
except ImportError:
    STEAM_AVAILABLE = False
    steam = None


class SteamAPI:
    """Steam API integration class for LithicRivers."""

    def __init__(self):
        """Initialize the Steam API client."""
        self.client = None
        self.is_connected = False

        if not STEAM_AVAILABLE:
            raise ImportError(
                "Steam API not available. Install with 'pip install steam[client]'"
            )

    def connect(self):
        """Connect to Steam API."""
        if not STEAM_AVAILABLE:
            return False

        try:
            self.client = steam.client.SteamClient()
            self.is_connected = True
            return True
        except Exception as e:
            print(f"Failed to connect to Steam API: {e}")
            self.is_connected = False
            return False

    def disconnect(self):
        """Disconnect from Steam API."""
        if self.client:
            self.client.close()
            self.client = None
        self.is_connected = False

    def is_available(self):
        """Check if Steam API is available."""
        return STEAM_AVAILABLE

    def get_user_info(self):
        """Get current user information if connected."""
        if not self.is_connected or not self.client:
            return None

        try:
            return {
                "steam_id": self.client.user.steam_id,
                "persona_name": self.client.user.persona_name,
                "online": self.client.user.online,
            }
        except Exception:
            return None


# Global Steam API instance
steam_api = None


def get_steam_api():
    """Get the global Steam API instance."""
    global steam_api
    if steam_api is None:
        try:
            steam_api = SteamAPI()
        except ImportError:
            steam_api = None
    return steam_api


def initialize_steam():
    """Initialize Steam API connection."""
    api = get_steam_api()
    if api:
        return api.connect()
    return False


def cleanup_steam():
    """Clean up Steam API connection."""
    global steam_api
    if steam_api:
        steam_api.disconnect()
        steam_api = None

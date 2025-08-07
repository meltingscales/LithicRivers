"""
Game Save Manager for LithicRivers

Handles all game saving and loading operations, including:
- Regular saves
- Snapshots for backup
- Save file management
- Error handling and recovery
"""

import logging
import pickle
import time
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional

from lithicrivers.game.core import Game
from lithicrivers.settings import DEFAULT_SEED, SAVES_FOLDER, SNAPSHOTS_FOLDER

logger = logging.getLogger(__name__)


class SaveMetadata:
    """Metadata for a save file."""

    def __init__(
        self,
        filename: str,
        timestamp: float,
        game_tick: int,
        player_name: str,
        world_seed: int,
    ):
        self.filename = filename
        self.timestamp = timestamp
        self.game_tick = game_tick
        self.player_name = player_name
        self.world_seed = world_seed
        self.readable_time = datetime.fromtimestamp(timestamp).strftime(
            "%Y-%m-%d %H:%M:%S"
        )

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for serialization."""
        return {
            "filename": self.filename,
            "timestamp": self.timestamp,
            "game_tick": self.game_tick,
            "player_name": self.player_name,
            "world_seed": self.world_seed,
            "readable_time": self.readable_time,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "SaveMetadata":
        """Create from dictionary."""
        return cls(
            data["filename"],
            data["timestamp"],
            data["game_tick"],
            data["player_name"],
            data["world_seed"],
        )


class SavedGameData:
    """
    Container for saved game data with embedded metadata.
    This makes save files completely self-contained and portable.
    """

    def __init__(self, game: "Game", metadata: SaveMetadata):
        self.game = game
        self.metadata = metadata
        self.save_version = "1.0"  # For future compatibility


class GameSaveManager:
    """
    Manages game saving and loading operations.

    Features:
    - Regular game saves with embedded metadata
    - Automatic snapshots for backup
    - Save file listing and metadata extraction
    - Error handling and recovery
    - Configurable save locations
    - Portable save files (metadata embedded in pickle)
    """

    def __init__(self):
        self.saves_folder = Path(SAVES_FOLDER)
        self.snapshots_folder = Path(SNAPSHOTS_FOLDER)
        self.default_save_name = "default-world.pkl"

        # Create directories if they don't exist
        self.saves_folder.mkdir(parents=True, exist_ok=True)
        self.snapshots_folder.mkdir(parents=True, exist_ok=True)

        # Track last snapshot tick for automatic snapshots
        self.last_snapshot_tick = 0
        self.snapshot_interval = 1_000_000  # Every 1 million ticks

    def save_game(
        self, game: "Game", save_name: Optional[str] = None, is_snapshot: bool = False
    ) -> bool:
        """
        Save a game to disk.

        Args:
            game: The Game instance to save
            save_name: Optional custom save name. If None, uses default.
            is_snapshot: Whether this is a snapshot save (goes to snapshots folder)

        Returns:
            bool: True if save was successful, False otherwise
        """
        try:
            # Determine save location and filename
            if is_snapshot:
                save_folder = self.snapshots_folder
                if save_name is None:
                    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
                    save_name = f"snapshot_{timestamp}_tick_{game.gametick}.pkl"
            else:
                save_folder = self.saves_folder
                if save_name is None:
                    save_name = self.default_save_name

            save_path = save_folder / save_name

            # Ensure the game is properly shut down before saving
            logger.info(f"Preparing game for save: {save_path}")
            game.shutdown()

            # Create metadata
            metadata = SaveMetadata(
                filename=save_name,
                timestamp=time.time(),
                game_tick=game.gametick,
                player_name=game.player.name,
                world_seed=game.world.seed,
            )

            # Create saved game data container with embedded metadata
            saved_data = SavedGameData(game, metadata)

            # Save the game with embedded metadata
            logger.info(f"Saving game to: {save_path}")
            with open(save_path, "wb") as f:
                pickle.dump(saved_data, f)

            logger.info(f"✅ Game saved successfully: {save_path}")
            return True

        except Exception as e:
            logger.error(f"❌ Failed to save game: {e}")
            return False

    def load_game(self, save_name: str = None) -> Optional["Game"]:
        """
        Load a game from a save file.

        Args:
            save_name: Name of the save file (defaults to default save)

        Returns:
            Game instance if successful, None otherwise
        """
        if save_name is None:
            save_name = self.default_save_name

        save_path = self.saves_folder / save_name

        if not save_path.exists():
            logger.info(f"Save file not found: {save_path}")
            return None

        try:
            logger.info(f"Loading game from: {save_path}")
            with open(save_path, "rb") as f:
                saved_data = pickle.load(f)

            # Extract game and metadata from SavedGameData container
            if not isinstance(saved_data, SavedGameData):
                raise ValueError(
                    f"Invalid save file format: expected SavedGameData, got {type(saved_data)}"
                )

            game = saved_data.game
            metadata = saved_data.metadata
            logger.info(
                f"Loaded game with metadata: {metadata.player_name}, tick {metadata.game_tick}"
            )

            # Ensure the game is marked as running
            game.running = True

            logger.info(f"✅ Game loaded successfully: {save_path}")
            return game

        except Exception as e:
            logger.error(f"❌ Failed to load game from {save_path}: {e}")
            return None

    def load_game_or_create_new(self) -> Optional["Game"]:
        """
        Load a game from a save file or create a new one if no save exists.

        Returns:
            Game instance if successful, None otherwise
        """
        return self.load_game() or self.create_new_game()

    def create_new_game(self) -> Optional["Game"]:
        """
        Create a new game.

        Returns:
            Game instance if successful, None otherwise
        """
        return Game(seed=DEFAULT_SEED, save_manager=self)

    def list_saves(self) -> List[SaveMetadata]:
        """
        List all available save files with their metadata.

        Returns:
            List of SaveMetadata objects
        """
        saves = []

        try:
            for save_file in self.saves_folder.glob("*.pkl"):
                metadata = self._extract_metadata_from_file(save_file)
                if metadata:
                    saves.append(metadata)

        except Exception as e:
            logger.error(f"Failed to list saves: {e}")

        # Sort by timestamp (newest first)
        saves.sort(key=lambda x: x.timestamp, reverse=True)
        return saves

    def delete_save(self, save_name: str) -> bool:
        """
        Delete a save file.

        Args:
            save_name: Name of the save file to delete

        Returns:
            bool: True if deletion was successful, False otherwise
        """
        try:
            save_path = self.saves_folder / save_name
            if save_path.exists():
                save_path.unlink()
                logger.info(f"Deleted save: {save_name}")
                return True
            else:
                logger.warning(f"Save file not found for deletion: {save_name}")
                return False
        except Exception as e:
            logger.error(f"Failed to delete save {save_name}: {e}")
            return False

    def create_snapshot(self, game: "Game") -> bool:
        """
        Create a snapshot save for backup purposes.

        Args:
            game: The Game instance to snapshot

        Returns:
            bool: True if snapshot was successful, False otherwise
        """
        return self.save_game(game, is_snapshot=True)

    def should_create_automatic_snapshot(self, current_tick: int) -> bool:
        """
        Check if an automatic snapshot should be created.

        Args:
            current_tick: Current game tick

        Returns:
            bool: True if a snapshot should be created
        """
        return (current_tick - self.last_snapshot_tick) >= self.snapshot_interval

    def create_automatic_snapshot_if_needed(self, game: "Game") -> bool:
        """
        Create an automatic snapshot if the interval has passed.

        Args:
            game: The Game instance to potentially snapshot

        Returns:
            bool: True if a snapshot was created, False otherwise
        """
        if self.should_create_automatic_snapshot(game.gametick):
            success = self.create_snapshot(game)
            if success:
                self.last_snapshot_tick = game.gametick
                logger.info(f"Created automatic snapshot at tick {game.gametick}")
            return success
        return False

    def _extract_metadata_from_file(self, save_path: Path) -> Optional[SaveMetadata]:
        """
        Extract metadata from a pickle file without fully loading the game.

        Args:
            save_path: Path to the save file

        Returns:
            SaveMetadata if successful, None otherwise
        """
        try:
            with open(save_path, "rb") as f:
                saved_data = pickle.load(f)

            # Extract metadata from SavedGameData container
            if not isinstance(saved_data, SavedGameData):
                raise ValueError(
                    f"Invalid save file format: expected SavedGameData, got {type(saved_data)}"
                )

            return saved_data.metadata

        except Exception as e:
            logger.error(f"Failed to extract metadata from {save_path}: {e}")
            # Fallback to file stats
            try:
                stat = save_path.stat()
                return SaveMetadata(
                    filename=save_path.name,
                    timestamp=stat.st_mtime,
                    game_tick=0,
                    player_name="Unknown",
                    world_seed="Unknown",
                )
            except Exception:
                return None

    def cleanup_old_snapshots(self, max_snapshots: int = 10) -> None:
        """
        Clean up old snapshot files, keeping only the most recent ones.

        Args:
            max_snapshots: Maximum number of snapshots to keep
        """
        try:
            snapshots = list(self.snapshots_folder.glob("snapshot_*.pkl"))
            if len(snapshots) <= max_snapshots:
                return

            # Sort by modification time (oldest first)
            snapshots.sort(key=lambda x: x.stat().st_mtime)

            # Delete oldest snapshots
            to_delete = snapshots[:-max_snapshots]
            for snapshot in to_delete:
                snapshot.unlink()
                logger.info(f"Cleaned up old snapshot: {snapshot.name}")

        except Exception as e:
            logger.error(f"Failed to cleanup old snapshots: {e}")

"""
Logging configuration for LithicRivers
Copyright (c) 2024 Henry Post. All rights reserved.
"""

import logging
import os.path
from typing import Optional

from lithicrivers.settings import LOGFILENAME, LOGGINGLEVEL


def setup_logging():
    """Configure logging for the application."""
    # Add custom log levels for verbose debugging
    logging.addLevelName(5, "SILLY")
    logging.addLevelName(1, "INSANE")

    # Remove existing log file if it exists
    if os.path.exists(LOGFILENAME):
        os.remove(LOGFILENAME)

    # Configure basic logging
    logging.basicConfig(
        filename=LOGFILENAME,
        level=LOGGINGLEVEL,
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    )


def get_logger(name: Optional[str] = None) -> logging.Logger:
    """Get a logger instance with the given name."""
    return logging.getLogger(name)

"""
Test package for LithicRivers.
"""

import logging
import os

# Configure logging for tests - set to WARNING to reduce noise
# This ensures debug messages from structure placement don't clutter test output
if os.environ.get("TESTING") == "1":
    logging.getLogger().setLevel(logging.WARNING)
    # Specifically silence the structure_generator logger during tests
    logging.getLogger("lithicrivers.structure_generator").setLevel(logging.ERROR)

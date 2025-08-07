#!/usr/bin/env python3
"""
Pre-generate test worlds for parallel test execution.

This script creates all the test world fixtures that will be used by unit tests,
ensuring they're available before parallel test execution begins. This eliminates
the need for complex locking mechanisms in SharedTestFixtures.
"""

import sys
import time
import logging
from pathlib import Path

# Add the project root to the path so we can import lithicrivers
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

from lithicrivers.test.test_fixtures import SharedTestFixtures
from lithicrivers.settings import DEFAULT_SEED

def setup_logging():
    """Configure logging for the script. Also logs to stdout."""
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s [%(levelname)s] %(message)s',
        datefmt='%H:%M:%S',
    )

def main():
    """Generate all test world fixtures."""
    setup_logging()
    
    logging.info("🏗️  Starting test world pre-generation...")
    start_time = time.time()
    
    # Create fixtures instance - this will trigger world generation
    # if worlds don't already exist
    fixtures = SharedTestFixtures()
    
    # List of all seeds we want to pre-generate
    # This includes the common seeds plus any others used in tests
    seeds_to_generate = fixtures.pregenerated_seeds
    logging.info(f"📦 Pre-generating worlds for {len(seeds_to_generate)} seeds...")
    
    generated_count = 0
    skipped_count = 0
    
    for seed in seeds_to_generate:
        if fixtures.does_save_exist(seed):
            logging.info(f"✅ Seed {seed}: Already exists, skipping")
            skipped_count += 1
        else:
            logging.info(f"🔨 Seed {seed}: Generating world...")
            
            # Generate and save the world
            # This uses the existing SharedTestFixtures methods
            world = fixtures.get_world(seed)
            
            logging.info(f"✅ Seed {seed}: Generated world with {len(world.data._chunks) if hasattr(world.data, '_chunks') else 0} chunks")
            generated_count += 1
    
    total_time = time.time() - start_time
    
    logging.info("🎉 Test world pre-generation complete!")
    logging.info(f"📊 Summary:")
    logging.info(f"   • Generated: {generated_count} new worlds")
    logging.info(f"   • Skipped: {skipped_count} existing worlds")
    logging.info(f"   • Total time: {total_time:.2f}s")
    logging.info(f"   • Average per world: {total_time/max(generated_count, 1):.2f}s")
    
    # Verify all worlds are accessible
    logging.info("🔍 Verifying all worlds are accessible...")
    for seed in seeds_to_generate:
        try:
            world = fixtures.get_world(seed)
            logging.info(f"✅ Seed {seed}: Verified accessible")
        except Exception as e:
            logging.error(f"❌ Seed {seed}: Failed to load - {e}")
            sys.exit(1)
    
    logging.info("🚀 All test worlds ready for parallel execution!")

if __name__ == "__main__":
    main()

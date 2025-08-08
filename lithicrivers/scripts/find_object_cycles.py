#!/usr/bin/env python3
"""
Script to analyze object cycles in the LithicRivers Game object using objgraph
and a networkx reference visualization. This script intentionally hard-requires
objgraph (no fallback) per project policy.
"""
import sys
import logging
import importlib

from lithicrivers.game.core import Game
import gc
from pprint import pprint
import objgraph

if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    print("\n=== LithicRivers Object Graph Cycle Analysis ===\n")

    # create a game
    game = Game.create(42)
    game.pregen_chunks(3)

    pprint(game.world.data.get_chunk_stats())

    # Objgraph-based cycle check (hard-required)
    print("\n[3] Objgraph cycle detection...")
    gc.collect()
    # Try to find a backreference chain from the Game object back to itself.
    # If found, we will render the chain PNG using graphviz via objgraph.
    chain = objgraph.find_backref_chain(game, predicate=lambda x: x is game, max_depth=20)
    if chain:
        print("Cycle back to Game detected. Writing game-cycle.png...")
        objgraph.show_chain(chain, filename="game-cycle.png")
        print("Saved as game-cycle.png")
    else:
        print("No backref chain from Game to itself within max_depth.")

    # Helpful type stats
    print("\n[4] Objgraph type stats (top 20)...")
    objgraph.show_most_common_types(limit=20)

    print("\nDone. Check game-cycle.png (if generated).")

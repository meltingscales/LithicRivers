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

    # Objgraph-based cycle checks (hard-required)
    print("\n[3] Objgraph cycle detection across key components (World, Player, Engine, WorldData)...")
    gc.collect()

    candidates = {
        "world": getattr(game, "world", None),
        "player": getattr(game, "player", None),
        "engine": getattr(game, "engine", None),
        "world_data": getattr(getattr(game, "world", None), "data", None),
    }

    # Remove Nones
    candidates = {k: v for k, v in candidates.items() if v is not None}

    any_found = False
    for name, obj in candidates.items():
        print(f"- Searching chain from {name} -> Game ...")
        try:
            chain = objgraph.find_backref_chain(obj, predicate=lambda x: x is game, max_depth=30)
        except RuntimeError as e:
            print(f"  Skipped {name} due to error: {e}")
            continue
        if chain:
            any_found = True
            out = f"cycle-{name}-to-game.png"
            objgraph.show_chain(chain, filename=out)
            print(f"  Found. Wrote {out}")
        else:
            print(f"  No chain from {name} to Game within max_depth.")

    for name, obj in candidates.items():
        print(f"- Searching chain from Game -> {name} ...")
        try:
            chain = objgraph.find_backref_chain(game, predicate=lambda x: x is obj, max_depth=30)
        except RuntimeError as e:
            print(f"  Skipped {name} due to error: {e}")
            continue
        if chain:
            any_found = True
            out = f"cycle-game-to-{name}.png"
            objgraph.show_chain(chain, filename=out)
            print(f"  Found. Wrote {out}")
        else:
            print(f"  No chain from Game to {name} within max_depth.")

    # Inbound references to Game (broad view)
    print("\n[4] Backrefs to Game (graph). This can be noisy; consider installing graphviz.)")
    try:
        objgraph.show_backrefs([game], max_depth=6, filename="game-backrefs.png")
        print("  Wrote game-backrefs.png")
    except Exception as e:
        print(f"  Skipped show_backrefs: {e}")

    # Helpful type stats
    print("\n[5] Objgraph type stats (top 20)...")
    objgraph.show_most_common_types(limit=20)

    if not any_found:
        print("\nNo non-trivial chains found. You may need to raise max_depth or add more candidate objects.")
    print("\nDone. Check generated PNGs (cycle-*, game-backrefs.png).")

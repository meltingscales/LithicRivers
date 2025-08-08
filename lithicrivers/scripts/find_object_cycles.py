#!/usr/bin/env python3
"""
Script to analyze object cycles in the LithicRivers Game object using objgraph
and a networkx reference visualization. This script intentionally hard-requires
objgraph (no fallback) per project policy.
"""
import sys
import logging
import importlib
import threading
import time
from collections import deque
from typing import Callable, Optional, Tuple, List

from lithicrivers.game.core import Game
import gc
from pprint import pprint
import objgraph

class Spinner:
    """Simple terminal spinner to indicate progress during long operations."""
    def __init__(self, prefix: str = "", interval: float = 0.1):
        self._stop = threading.Event()
        self._thread = None
        self.prefix = prefix
        self.interval = interval

    def start(self):
        if self._thread is not None:
            return
        self._thread = threading.Thread(target=self._run, daemon=True)
        self._thread.start()
        return self

    def stop(self):
        self._stop.set()
        if self._thread is not None:
            self._thread.join(timeout=1.0)
        # Ensure spinner line is cleared
        sys.stdout.write("\r" + " " * 80 + "\r")
        sys.stdout.flush()

    def _run(self):
        glyphs = "|/-\\"
        i = 0
        while not self._stop.is_set():
            ch = glyphs[i % len(glyphs)]
            sys.stdout.write(f"\r{self.prefix} {ch}")
            sys.stdout.flush()
            time.sleep(self.interval)
            i += 1


def find_backref_chain_with_count(
    start_obj: object,
    predicate: Callable[[object], bool],
    max_depth: int = 30,
) -> Tuple[List[object], int]:
    """
    Instrumented BFS over backreferences using gc.get_referrers to find a chain
    to an object matching predicate. Returns (chain, traversals_count).

    The chain is a list from the matching object back to start_obj (same order
    as objgraph.find_backref_chain for compatibility with show_chain).
    """
    seen: set[int] = set()
    parent: dict[int, Optional[int]] = {}
    node_obj: dict[int, object] = {}
    q = deque([(start_obj, 0)])
    start_id = id(start_obj)
    parent[start_id] = None
    node_obj[start_id] = start_obj
    traversals = 0

    while q:
        obj, depth = q.popleft()
        obj_id = id(obj)
        if obj_id in seen:
            continue
        seen.add(obj_id)
        if depth >= max_depth:
            continue
        # Explore inbound refs
        try:
            refs = gc.get_referrers(obj)
        except Exception:
            refs = []
        for ref in refs:
            traversals += 1
            rid = id(ref)
            if rid in seen or rid == obj_id:
                continue
            # Skip frames to reduce noise/explosions
            if ref.__class__.__name__ in {"frame", "cell"}:
                continue
            node_obj[rid] = ref
            parent[rid] = obj_id
            if predicate(ref):
                # Reconstruct chain from ref -> ... -> start_obj
                chain: List[object] = []
                cur: Optional[int] = rid
                while cur is not None:
                    chain.append(node_obj[cur])
                    cur = parent.get(cur)
                return chain, traversals
            q.append((ref, depth + 1))
    return [], traversals

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
    total = len(candidates)
    for idx, (name, obj) in enumerate(candidates.items(), start=1):
        print(f"- [{idx}/{total}] Searching chain from {name} -> Game ...")
        spinner = Spinner(prefix=f"  find_backref_chain({name} -> Game)").start()
        try:
            chain, traversals = find_backref_chain_with_count(obj, predicate=lambda x: x is game, max_depth=30)
        except RuntimeError as e:
            spinner.stop()
            print(f"  Skipped {name} due to error: {e}")
            continue
        finally:
            spinner.stop()
        if chain:
            any_found = True
            out = f"cycle-{name}-to-game.png"
            print("  Rendering chain PNG...")
            objgraph.show_chain(chain, filename=out)
            print(f"  Found. Wrote {out}. Traversals: {traversals}")
        else:
            print(f"  No chain from {name} to Game within max_depth. Traversals: {traversals}")

    for idx, (name, obj) in enumerate(candidates.items(), start=1):
        print(f"- [{idx}/{total}] Searching chain from Game -> {name} ...")
        spinner = Spinner(prefix=f"  find_backref_chain(Game -> {name})").start()
        try:
            chain, traversals = find_backref_chain_with_count(game, predicate=lambda x: x is obj, max_depth=30)
        except RuntimeError as e:
            spinner.stop()
            print(f"  Skipped {name} due to error: {e}")
            continue
        finally:
            spinner.stop()
        if chain:
            any_found = True
            out = f"cycle-game-to-{name}.png"
            print("  Rendering chain PNG...")
            objgraph.show_chain(chain, filename=out)
            print(f"  Found. Wrote {out}. Traversals: {traversals}")
        else:
            print(f"  No chain from Game to {name} within max_depth. Traversals: {traversals}")

    # Inbound references to Game (broad view)
    print("\n[4] Backrefs to Game (graph). This can be noisy; consider installing graphviz.)")
    try:
        spinner = Spinner(prefix="  show_backrefs(Game)").start()
        objgraph.show_backrefs([game], max_depth=6, filename="game-backrefs.png")
    finally:
        spinner.stop()
    print("  Wrote game-backrefs.png")

    # Helpful type stats
    print("\n[5] Objgraph type stats (top 20)...")
    objgraph.show_most_common_types(limit=20)

    if not any_found:
        print("\nNo non-trivial chains found. You may need to raise max_depth or add more candidate objects.")
    print("\nDone. Check generated PNGs (cycle-*, game-backrefs.png).")

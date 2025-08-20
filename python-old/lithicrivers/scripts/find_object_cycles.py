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
import inspect
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
        self._status = ""
        self._lock = threading.Lock()

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
            with self._lock:
                status = self._status
            sys.stdout.write(f"\r{self.prefix} {ch}  {status}")
            sys.stdout.flush()
            time.sleep(self.interval)
            i += 1

    def set_status(self, text: str):
        with self._lock:
            self._status = text


def find_backref_chain_with_count(
    start_obj: object,
    predicate: Callable[[object], bool],
    max_depth: int = 30,
    on_step: Optional[Callable[[int], None]] = None,
    report_every: int = 500,
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
            if on_step is not None and (traversals % report_every == 0):
                try:
                    on_step(traversals)
                except Exception:
                    pass
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


def run_chain_round(
    starts: dict[str, object],
    game: object,
    *,
    reverse: bool = False,
    max_depth: int = 30,
) -> bool:
    """Run a single round of chain searches.

    If reverse is False, searches each start -> Game.
    If reverse is True, searches Game -> each start.
    Returns True if any chain was found in this round.
    """
    any_found = False
    total = len(starts)
    for idx, (name, obj) in enumerate(starts.items(), start=1):
        if reverse:
            print(f"- [{idx}/{total}] Searching chain from Game -> {name} ...")
            spinner = Spinner(prefix=f"  find_backref_chain(Game -> {name})").start()
            try:
                chain, traversals = find_backref_chain_with_count(
                    game,
                    predicate=lambda x, target=obj: x is target,
                    max_depth=max_depth,
                    on_step=lambda n: spinner.set_status(f"{n} traversals"),
                )
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
        else:
            print(f"- [{idx}/{total}] Searching chain from {name} -> Game ...")
            spinner = Spinner(prefix=f"  find_backref_chain({name} -> Game)").start()
            try:
                chain, traversals = find_backref_chain_with_count(
                    obj,
                    predicate=lambda x, target=game: x is target,
                    max_depth=max_depth,
                    on_step=lambda n: spinner.set_status(f"{n} traversals"),
                )
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
    return any_found

if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    print("\n=== LithicRivers Object Graph Cycle Analysis ===\n")

    # create a game
    game = Game.create(42)
    game.pregen_chunks(1)  # Pre-generate a single chunk for testing

    pprint(game.world.data.get_chunk_stats())

    # Objgraph-based cycle checks (hard-required)
    print("\n[3] Objgraph cycle detection across key components (World, Player, Engine, WorldData)...")
    gc.collect()

    def _safe_get(obj, attr):
        try:
            return getattr(obj, attr)
        except Exception:
            return None

    def build_candidates(game: object, *, auto_discover: bool = True, auto_limit: int = 20) -> dict[str, object]:
        c: dict[str, object] = {}
        # Core
        base = {
            "game": game,
            "world": _safe_get(game, "world"),
            "player": _safe_get(game, "player"),
            "engine": _safe_get(game, "engine"),
            "world_data": _safe_get(_safe_get(game, "world"), "data"),
            # High-probability runtime-only refs
            "message_log": _safe_get(game, "message_log"),
            "save_manager": _safe_get(game, "save_manager"),
            # Common caches/managers
            "chunk_cache": _safe_get(_safe_get(_safe_get(game, "world"), "data"), "chunk_cache"),
            "entities": _safe_get(_safe_get(game, "world"), "entities"),
            "fluids": _safe_get(game, "fluids") or _safe_get(_safe_get(game, "world"), "fluids"),
            "npc_manager": _safe_get(game, "npc_manager"),
            # Worldgen/managers if present
            "structure_manager": _safe_get(game, "structure_manager") or _safe_get(_safe_get(game, "world"), "structure_manager"),
            "procedural_generator": _safe_get(game, "procedural_generator") or _safe_get(_safe_get(game, "world"), "procedural_generator"),
        }
        for k, v in base.items():
            if v is not None:
                c[k] = v

        if auto_discover:
            def include_obj(name: str, obj: object) -> bool:
                if obj is None:
                    return False
                if name in c:
                    return False
                t = type(obj)
                mod = getattr(t, "__module__", "")
                if not isinstance(obj, (int, float, bool, str)) and mod.startswith("lithicrivers"):
                    return True
                return False

            scanned = 0
            owners = {
                "game": game,
                "world": _safe_get(game, "world"),
                "engine": _safe_get(game, "engine"),
                "player": _safe_get(game, "player"),
                "world_data": _safe_get(_safe_get(game, "world"), "data"),
            }
            for owner_name, owner in owners.items():
                if owner is None:
                    continue
                for attr in dir(owner):
                    if attr.startswith("_"):
                        continue
                    # Avoid methods and properties with side effects by try/except
                    try:
                        val = getattr(owner, attr)
                    except Exception:
                        continue
                    # Skip callables and modules
                    if callable(val) or inspect.ismodule(val):
                        continue
                    name = f"{owner_name}.{attr}"
                    if include_obj(name, val):
                        c[name] = val
                        scanned += 1
                        if scanned >= auto_limit:
                            break
                if scanned >= auto_limit:
                    break

        return c

    candidates = build_candidates(game, auto_discover=True, auto_limit=20)

    any_found = False
    any_found |= run_chain_round(candidates, game, reverse=False, max_depth=30)
    any_found |= run_chain_round(candidates, game, reverse=True, max_depth=30)

    if not any_found:
        print("\nNo non-trivial chains found. You may need to raise max_depth or add more candidate objects.")
    print("\nDone. Check generated PNGs (cycle-*, game-backrefs.png).")

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

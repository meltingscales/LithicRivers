#!/usr/bin/env python3
"""
Script to analyze object cycles in the LithicRivers Game object using objgraph and a custom cycle finder.
"""
import sys
import logging
import importlib

import networkx as nx
import matplotlib.pyplot as plt
from lithicrivers.game.core import Game
import gc
from pprint import pprint

def find_cycles(obj, seen=None, path=None):
    """Recursively traverse the object graph to find cycles."""
    if seen is None:
        seen = set()
    if path is None:
        path = []
    obj_id = id(obj)
    if obj_id in seen:
        print("Cycle detected! Path:", " -> ".join(path))
        return True
    seen.add(obj_id)
    path.append(repr(type(obj)))
    # Only traverse user objects (skip builtins)
    if hasattr(obj, '__dict__'):
        for attr, value in obj.__dict__.items():
            if isinstance(value, (list, tuple, set)):
                for item in value:
                    if find_cycles(item, seen, path[:] + [f"{attr}[{type(item).__name__}]"]):
                        return True
            elif isinstance(value, dict):
                for k, v in value.items():
                    if find_cycles(v, seen, path[:] + [f"{attr}[{repr(k)}]"]):
                        return True
            elif hasattr(value, "__dict__"):
                if find_cycles(value, seen, path[:] + [attr]):
                    return True
    return False

if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    print("\n=== LithicRivers Object Graph Cycle Analysis ===\n")

    # create a game
    game = Game.create(42)
    game.pregen_chunks(3)

    pprint(game.world.data.get_chunk_stats())

    # Build a reference graph using networkx
    print("\n[1] Building object reference graph with networkx...")
    G = nx.DiGraph()
    node_labels = {}

    def add_edges(obj, parent_id=None, depth=0, max_depth=3, seen=None):
        if seen is None:
            seen = set()
        if depth > max_depth:
            return
        obj_id = id(obj)
        if obj_id in seen:
            return
        seen.add(obj_id)
        label = f"{type(obj).__name__}\n{id(obj)}"
        node_labels[obj_id] = label
        G.add_node(obj_id)
        if parent_id is not None:
            G.add_edge(parent_id, obj_id)
        if hasattr(obj, "__dict__"):
            for attr, value in obj.__dict__.items():
                if isinstance(value, (list, tuple, set)):
                    for item in value:
                        add_edges(item, obj_id, depth+1, max_depth, seen)
                elif isinstance(value, dict):
                    for k, v in value.items():
                        add_edges(v, obj_id, depth+1, max_depth, seen)
                elif hasattr(value, "__dict__"):
                    add_edges(value, obj_id, depth+1, max_depth, seen)

    add_edges(game, None, 0, 3)

    print(f"Reference graph has {G.number_of_nodes()} nodes and {G.number_of_edges()} edges.")
    print("\n[2] Drawing reference graph as game-networkx.png (matplotlib)...")
    plt.figure(figsize=(12, 8))
    pos = nx.spring_layout(G, k=0.3, iterations=20)
    nx.draw(G, pos, with_labels=False, node_size=200, alpha=0.7, arrows=True)
    nx.draw_networkx_labels(G, pos, labels=node_labels, font_size=7)
    plt.title("Game Object Reference Graph (truncated)")
    plt.savefig("game-networkx.png", bbox_inches="tight")
    plt.close()
    print("Saved as game-networkx.png")

    print("\n[3] Running custom cycle finder (may be slow)...")
    found_cycle = find_cycles(game)
    if not found_cycle:
        print("No cycles detected by custom finder (within traversal limits).")
    print("\nDone. Check game-networkx.png for the visual reference graph.")

"""Print a smaller steamcmd config.vdf for a GitHub secret (48 KB at most).

    --sizes           list every block with its size (names only, never values)
    --drop A,B,...    print the file without the blocks with those names, at any depth

Usage:
  python3 -I trim_steam_config.py config.vdf --sizes
  python3 -I trim_steam_config.py config.vdf --drop depots,apps | gh secret set STEAM_CONFIG_VDF
"""

import re
import sys



def tokens(text):
    for m in re.finditer(r'"((?:[^"\\]|\\.)*)"|([{}])', text):
        yield ("str", m.group(1)) if m.group(2) is None else (m.group(2), None)


def parse(it):
    """A block as a list of (key, value) where value is a string or a nested list."""
    items = []
    for kind, val in it:
        if kind == "}":
            return items
        if kind != "str":
            raise ValueError("unexpected {")
        kind2, val2 = next(it)
        if kind2 == "{":
            items.append((val, parse(it)))
        elif kind2 == "str":
            items.append((val, val2))
        else:
            raise ValueError("unexpected }")
    return items


def find(items, key):
    for k, v in items:
        if k.lower() == key.lower() and isinstance(v, list):
            return v
    return None


def emit(items, depth=0):
    out = []
    pad = "\t" * depth
    for k, v in items:
        if isinstance(v, list):
            out.append(f'{pad}"{k}"\n{pad}{{\n{emit(v, depth + 1)}{pad}}}\n')
        else:
            out.append(f'{pad}"{k}"\t\t"{v}"\n')
    return "".join(out)


def sizes(items, depth=0):
    for k, v in items:
        if isinstance(v, list):
            print(f"{'  ' * depth}{k}: {len(emit(v))} bytes")
            if depth < 4:
                sizes(v, depth + 1)


def drop(items, names):
    return [
        (k, drop(v, names) if isinstance(v, list) else v)
        for k, v in items
        if not (isinstance(v, list) and k.lower() in names)
    ]


def main():
    text = open(sys.argv[1], encoding="utf-8").read()
    root = parse(tokens(text))
    if sys.argv[2:3] == ["--sizes"]:
        sizes(root)
        return
    if sys.argv[2:3] != ["--drop"] or len(sys.argv) < 4:
        sys.exit(__doc__)
    names = {n.strip().lower() for n in sys.argv[3].split(",")}
    trimmed = drop(root, names)
    if "ConnectCache" not in emit(trimmed):
        sys.exit("no ConnectCache block left: don't drop it")
    result = emit(trimmed)
    sys.stderr.write(f"trimmed config.vdf: {len(result)} bytes (was {len(text)})\n")
    if len(result) > 48 * 1024:
        sys.exit("still over 48 KB: drop more blocks (see --sizes)")
    sys.stdout.write(result)


if __name__ == "__main__":
    main()

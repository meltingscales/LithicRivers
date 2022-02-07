from typing import List, Tuple

from settings import DEFAULT_SIZE


def print_world(juiceWorld: List[List[str]], size: Tuple[int, int] = DEFAULT_SIZE, borderchar: str = None) -> None:
    width, height = size
    for y in range(0, height):
        row = juiceWorld[y]

        if (y == 0) and borderchar:
            print((borderchar * 2) + (borderchar * width))

        for x in range(0, width):
            tile = row[x]

            if (x == 0) and borderchar:
                print(borderchar, end='')

            print(tile, end='')

            if (x == (width - 1)) and borderchar:
                print(borderchar, end='')

        print("\n", end='')

        if (y == (height - 1)) and borderchar:
            print((borderchar * 2) + (borderchar * width))

    return None
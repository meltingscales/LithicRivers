# This is a sample Python script.

# Press Ctrl+F5 to execute it or replace it with your code.
# Press Double Shift to search everywhere for classes, files, tool windows, actions, and settings.
import random
from typing import List, Tuple

DEFAULT_SIZE = (15, 20)


def gen_tile() -> str:
    i = random.randint(1, 10)

    if i >= 10:
        return 'i'

    return ' '


def gen_world(size=DEFAULT_SIZE) -> List[List[str]]:
    width, height = size
    resultworld = []
    for y in range(0, height):
        row = []
        for x in range(0, width):
            row.append(gen_tile())
        resultworld.append(row)
    return resultworld


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


world = gen_world()
print_world(world, size=DEFAULT_SIZE, borderchar='#')


def print_hi(name):
    # Use a breakpoint in the code line below to debug your script.
    print(f'Hi, {name}')  # Press F9 to toggle the breakpoint.


# Press the green button in the gutter to run the script.
if __name__ == '__main__':
    print_hi('PyCharm')

# See PyCharm help at https://www.jetbrains.com/help/pycharm/

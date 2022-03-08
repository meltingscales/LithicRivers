"""
This util exists to unify TUI styles and make the game look + feel cohesive.
"""

# class Colors:
#     BLUE = 'adsf'
#
#
# def color(text: str, colorName: str) -> str:
#     col = Colors.__getattribute__(name=colorName.upper())
#     raise NotImplementedError("color NYI, im lazy")
from typing import List


def presenting(text) -> str:
    return "~ {} ~".format(text)


def render_int_tuple(tups: List[int], places=2) -> str:
    fstr = ""

    for i in range(0, len(tups)):
        fstr += "{:" + str(places) + "d}"

        if i < (len(tups) - 1):
            fstr += ','

    return fstr.format(*tups)


def associated(text1, text2):
    return "{} => {}".format(text1, text2)


def emphasizing(text) -> str:
    return "[ {} ]".format(text)


def list_label(text, width=5, align='>') -> str:
    return "-[{:{}{}s}]: ".format(text, align, width)

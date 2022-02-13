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


def presenting(text) -> str:
    return "~ {} ~".format(text)


def associated(text1, text2):
    return "{} => {}".format(text1, text2)


def emphasizing(text) -> str:
    return "[ {} ]".format(text)


def list_label(text, width=5, align='>') -> str:
    return "-[{:{}{}s}]: ".format(text, align, width)

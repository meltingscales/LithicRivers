# This is a sample Python script.

# Press Ctrl+F5 to execute it or replace it with your code.
# Press Double Shift to search everywhere for classes, files, tool windows, actions, and settings.
import random
import sys
from pprint import pprint
from typing import List, Tuple

import asciimatics.widgets
from asciimatics.event import KeyboardEvent
from asciimatics.exceptions import ResizeScreenError, NextScene, StopApplication
from asciimatics.scene import Scene
from asciimatics.screen import Screen
from asciimatics.widgets import Frame, Layout, Divider, Button

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


def raise_(ex):
    """Because we can't use raise in lambda for some reason..."""
    raise ex


class TabButtons(Layout):
    def __init__(self, frame, active_tab_idx):
        cols = [1, 1, 1, 1, 1]

        super().__init__(cols)

        self._frame = frame

        for i, _ in enumerate(cols):
            self.add_widget(Divider(), i)

        buttons = [
            Button("Btn1", lambda: raise_(NextScene("Tab1"))),
            Button("Btn2", lambda: raise_(NextScene("Tab2"))),
            Button("Btn3", lambda: raise_(NextScene("Tab3"))),
            Button("Btn4", lambda: raise_(NextScene("Tab4"))),
            Button("Quit", lambda: (print("Bye!"), raise_(StopApplication("Quit"))))
        ]

        for i, button in enumerate(buttons):
            self.add_widget(button, i)

        buttons[active_tab_idx].disabled = True


class RootPage(Frame):
    def __init__(self, screen):
        super().__init__(screen,
                         screen.height,
                         screen.width,
                         can_scroll=False,
                         title="Root Page")

        layout1 = Layout([1], fill_frame=True)

        self.add_layout(layout1)

        text1 = asciimatics.widgets.Text(name="text1", label="hello - tile generated = [{}]".format(gen_tile()))
        layout1.add_widget(text1)

        layout2 = TabButtons(self, 0)
        self.add_layout(layout2)
        self.fix()


class AlphaPage(Frame):
    def __init__(self, screen):
        super().__init__(screen,
                         screen.height,
                         screen.width,
                         can_scroll=False,
                         title="Alpha Page")
        layout1 = Layout([1], fill_frame=True)
        self.add_layout(layout1)
        # add your widgets here

        layout2 = TabButtons(self, 1)
        self.add_layout(layout2)
        self.fix()


class BravoPage(Frame):
    def __init__(self, screen):
        super().__init__(screen,
                         screen.height,
                         screen.width,
                         can_scroll=False,
                         title="Bravo Page")
        layout1 = Layout([1], fill_frame=True)
        self.add_layout(layout1)
        # add your widgets here

        layout2 = TabButtons(self, 2)
        self.add_layout(layout2)
        self.fix()


class CharliePage(Frame):
    def __init__(self, screen):
        super().__init__(screen,
                         screen.height,
                         screen.width,
                         can_scroll=False,
                         title="Charlie Page")
        layout1 = Layout([1], fill_frame=True)
        self.add_layout(layout1)
        # add your widgets here

        layout2 = TabButtons(self, 3)
        self.add_layout(layout2)
        self.fix()


def demo(screen: Screen, scene: Scene):
    scenes = [
        Scene([RootPage(screen)], -1, name="Tab1"),
        Scene([AlphaPage(screen)], -1, name="Tab2"),
        Scene([BravoPage(screen)], -1, name="Tab3"),
        Scene([CharliePage(screen)], -1, name="Tab4"),
    ]

    def handleEvent(event: KeyboardEvent):
        daChar=chr(event.key_code)
        print("WOW! Event = {}".format(daChar))
        pprint(event)
        screen.set_title("HOLD UP, YOU PRESSING "+daChar+"?")

    screen.play(scenes, stop_on_resize=True, start_scene=scene, allow_int=True, unhandled_input=handleEvent)


# Press the green button in the gutter to run the script.
if __name__ == '__main__':
    print('wow its PyCharm!')

    last_scene = None
    while True:
        try:
            Screen.wrapper(demo, catch_interrupt=True, arguments=[last_scene])
            sys.exit(0)
        except ResizeScreenError as e:
            last_scene = e.scene

# See PyCharm help at https://www.jetbrains.com/help/pycharm/

# This is a sample Python script.

# Press Ctrl+F5 to execute it or replace it with your code.
# Press Double Shift to search everywhere for classes, files, tool windows, actions, and settings.
import io
import sys
from typing import List, Tuple, Union

import asciimatics.widgets
import numpy
from asciimatics.effects import Effect
from asciimatics.event import KeyboardEvent, MouseEvent
from asciimatics.exceptions import ResizeScreenError, NextScene, StopApplication
from asciimatics.scene import Scene
from asciimatics.screen import Screen
from asciimatics.widgets import Frame, Layout, Divider, Button

from settings import DEFAULT_SIZE


def gen_tile(choices=None, weights=None) -> str:
    if choices is None:
        choices = ['@', ',', '.']

    if weights is None:
        weights = [5, 15, 100]

    if len(weights) != len(choices):
        raise ValueError(f"Weights and choices for {gen_tile.__name__}() must be the same length!")

    weights = numpy.asarray(weights)

    normalizedWeights = weights

    if weights.sum() != 1:
        normalizedWeights = weights / weights.sum()

    return numpy.random.choice(choices, p=normalizedWeights)


def raise_(ex):
    """Because we can't use raise in lambda for some reason..."""
    raise ex


class Player:
    def __init__(self):
        self.x = 0
        self.y = 0
        self.displaySymbol = '$'
        self.health = 100
        self.stamina = 100

    def move(self, vecoffset: Tuple[int, int]):
        xoffset, yoffset = vecoffset

        self.x += xoffset
        self.y += yoffset

    def moveUp(self):
        self.move((0, 1))

    def moveDown(self):
        self.move((0, -1))

    def moveLeft(self):
        self.move((-1, 0))

    def moveRight(self):
        self.move((1, 0))


class World:

    @staticmethod
    def gen_world(size: Tuple[int, int]) -> List[List[str]]:
        width, height = size
        resultworld = []
        for y in range(0, height):
            row = []
            for x in range(0, width):
                row.append(gen_tile())
            resultworld.append(row)
        return resultworld

    def __init__(self, size=DEFAULT_SIZE):
        self.size = size
        self.worlddata = World.gen_world(size)

    def print_world(self, borderchar: str = None) -> str:

        if borderchar == '':
            borderchar = None

        daFile = io.StringIO()

        size = self.size
        juiceWorld = self.worlddata

        width, height = size
        for y in range(0, height):
            row = juiceWorld[y]

            if (y == 0) and borderchar:
                print((borderchar * 2) + (borderchar * width), file=daFile)

            for x in range(0, width):
                tile = row[x]

                if (x == 0) and borderchar:
                    print(borderchar, end='', file=daFile)

                print(tile, end='', file=daFile)

                if (x == (width - 1)) and borderchar:
                    print(borderchar, end='', file=daFile)

            print("\n", end='', file=daFile)

            if (y == (height - 1)) and borderchar:
                print((borderchar * 2) + (borderchar * width), file=daFile)

        ret = daFile.getvalue()
        daFile.close()
        return ret


class Game:
    def __init__(self, player=None, world=None):

        if player is None:
            player = Player()

        if world is None:
            world = World()

        self.player = player
        self.world = world


# Global game object...
GAME = Game()


class TabButtons(Layout):
    def __init__(self, frame, active_tab_idx):
        cols = [1, 1, 1, 1, 1]

        super().__init__(cols)

        self._frame = frame

        for i, _ in enumerate(cols):
            self.add_widget(Divider(), i)

        buttons = [
            Button("Root Page", lambda: raise_(NextScene("RootPage"))),
            Button("Alpha Page", lambda: raise_(NextScene("AlphaPage"))),
            Button("Bravo Page", lambda: raise_(NextScene("BravoPage"))),
            Button("Charlie Page", lambda: raise_(NextScene("CharliePage"))),
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

        self.textTileGen = asciimatics.widgets.Text(name="textTileGen",
                                                    label="[focus input]", readonly=True)
        layout1.add_widget(self.textTileGen)

        self.labelFoo = asciimatics.widgets.Label(name="labelFoo", label='foo :)')
        layout1.add_widget(self.labelFoo)

        self.textBoxGameRender = asciimatics.widgets.TextBox(
            name="textBoxGameRender",
            label="World:",
            height=20,
            readonly=True,
        )
        self.textBoxGameRender.value = GAME.world.print_world(borderchar='')
        layout1.add_widget(self.textBoxGameRender)

        layoutButtons = TabButtons(self, 0)
        self.add_layout(layoutButtons)
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
        Scene([RootPage(screen)], -1, name="RootPage"),
        Scene([AlphaPage(screen)], -1, name="AlphaPage"),
        Scene([BravoPage(screen)], -1, name="BravoPage"),
        Scene([CharliePage(screen)], -1, name="CharliePage"),
    ]

    def handleEvent(event: Union[KeyboardEvent, MouseEvent]):
        if not isinstance(event, KeyboardEvent):
            print("not keyboard event, ignoring... - {}".format(event))
            return
        event: KeyboardEvent

        daChar = chr(event.key_code)
        # pprint(event)

        daScene: Scene = screen.current_scene
        daEffects: List[Effect] = daScene.effects

        if len(daEffects) <= 0:
            print("No effects ;_;")
            return

        maybeDaRootPage: RootPage = daEffects[0]

        if maybeDaRootPage.title.strip() == 'Root Page':
            daRootPage = maybeDaRootPage
            # screen.set_title("HOLD UP, YOU PRESSING " + daChar + "?")
            daRootPage.labelFoo.text = f"pressing {daChar}?"
            # daRootPage.textBoxGameRender.value = GAME.world.print_world()
            daRootPage.textBoxGameRender.value = "test\nit\nout"
        else:
            print("Not supposed to handle " + maybeDaRootPage.title)

    screen.play(scenes, stop_on_resize=True, start_scene=scene, allow_int=True, unhandled_input=handleEvent)


# Press the green button in the gutter to run the script.
if __name__ == '__main__':
    print('wow its PyCharm!')

    print(GAME.world.print_world())

    last_scene = None
    while True:
        try:
            Screen.wrapper(demo, catch_interrupt=True, arguments=[last_scene])
            sys.exit(0)
        except ResizeScreenError as e:
            last_scene = e.scene

# See PyCharm help at https://www.jetbrains.com/help/pycharm/

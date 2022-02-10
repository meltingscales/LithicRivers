# This is a sample Python script.

# Press Ctrl+F5 to execute it or replace it with your code.
# Press Double Shift to search everywhere for classes, files, tool windows, actions, and settings.
import logging
import os.path
import sys
from typing import List, Tuple, Union, Dict

import asciimatics.widgets
import numpy
from asciimatics.effects import Effect
from asciimatics.event import KeyboardEvent, MouseEvent
from asciimatics.exceptions import ResizeScreenError, NextScene, StopApplication
from asciimatics.scene import Scene
from asciimatics.screen import Screen, Canvas
from asciimatics.widgets import Frame, Layout, Divider, Button, _split_text

from settings import DEFAULT_SIZE, LOGFILENAME


def raise_(ex):
    """Because we can't use raise in lambda for some reason..."""
    raise ex


if os.path.exists(LOGFILENAME):
    os.remove(LOGFILENAME)

logging.basicConfig(filename=LOGFILENAME, level=logging.DEBUG)


class Item:
    def __init__(self, name, charsheet: List[str] = None):
        self.name = name
        self.charsheet = charsheet

    def render(self, scale=1):
        return self.charsheet[scale - 1]


class Items:
    """
    A bunch of default items.
    """

    @staticmethod
    def Rock():
        return Item("Rock",
                    charsheet=['*'])

    @staticmethod
    def Gold_Nugget():
        return Item("Gold Nugget",
                    charsheet=['c'])

    @staticmethod
    def Stick():
        return Item("Stick",
                    charsheet=['\\'])


class Tiles:
    """
    A bunch of default tiles.
    """

    @staticmethod
    def Dirt():
        return Tile('Dirt',
                    [',', ',.\n'
                          '.,', ',.,\n'
                                '.,.\n'
                                ',.,'],
                    {0.99: Items.Rock(),
                     0.01: Items.Gold_Nugget()})

    @staticmethod
    def Tree():
        return Tile('Tree',
                    ['t', '/\\\n'
                          '||', '/|\\\n'
                                ';|;\n'
                                '/|\\\n'],
                    {1.00: Items.Stick()})


class Tile:
    def __init__(self, name: str, charsheet: List[str] = None, drops: Dict[float, Item] = None):
        self.name = name
        self.charsheet = charsheet
        self.drops = drops

    def render(self, scale=1):
        return self.charsheet[scale - 1]


def gen_tile(choices: List[Tile] = None, weights: List[int] = None) -> Tile:
    if choices is None:
        choices = [Tiles.Tree(),
                   Tiles.Dirt()]

    if weights is None:
        weights = [5, 100]

    if len(weights) != len(choices):
        ve = ValueError(f"Weights and choices for {gen_tile.__name__}() must be the same length!")
        logging.error(ve)
        raise ve

    weights = numpy.asarray(weights)

    normalizedWeights = weights

    if weights.sum() != 1:
        normalizedWeights = weights / weights.sum()

    return numpy.random.choice(choices, p=normalizedWeights)


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

    def calcOffset(self, vec: Tuple[int, int]) -> Tuple[int, int]:
        """Where would I move, if I did move?"""
        xoffset, yoffset = vec

        return (
            self.x + xoffset,
            self.y + yoffset
        )

    def moveUp(self):
        self.move((0, 1))

    def moveDown(self):
        self.move((0, -1))

    def moveLeft(self):
        self.move((-1, 0))

    def moveRight(self):
        self.move((1, 0))

    def render(self, scale):
        return '$'


class World:

    def get_height(self):
        return self.size[1]

    def get_width(self):
        return self.size[0]

    @staticmethod
    def gen_random_world(size: Tuple[int, int]) -> List[List[Tile]]:
        width, height = size
        resultworld = []
        for y in range(0, height):
            row = []
            for x in range(0, width):
                row.append(gen_tile())
            resultworld.append(row)
        return resultworld

    def __init__(self, name="Gaia", size=DEFAULT_SIZE):
        self.size = size
        self.name = name
        self.worlddata = World.gen_random_world(size)
        self.gametick = 0


class Game:
    def __init__(self, player: Player = None, world: World = None):

        if player is None:
            player = Player()

        if world is None:
            world = World()

        self.player = player
        self.world = world

    def render_world(self, scale: int = 1) -> List[List[str]]:
        r"""
        :param scale: Scaling for sprites. <pre><code>
        1 = x, 2 = \/, 3 = \ /, etc.
                   /\       x
                           / \
        :return:
        """

        ret = []

        for row in self.world.worlddata:
            retrow = []
            for tile in row:
                retrow.append(tile.render(scale=scale))  # TODO: will break with scale>2... we need to print to a buffer
            ret.append(retrow)

        # TODO: Assert ret is well-formed

        logging.info("player x,y={:2d},{:2d}".format(self.player.x, self.player.y))
        ret[self.player.y][self.player.x] = self.player.render(scale=scale)

        return ret

    def move_player(self, vec):
        possiblePosition = self.player.calcOffset(vec)

        # check bounds
        if (
                (possiblePosition[0] < 0) or
                (possiblePosition[1] < 0) or
                (possiblePosition[0] >= self.world.get_width()) or
                (possiblePosition[1] >= self.world.get_height())
        ):
            logging.debug("Tried to move OOB! {} would have resulted in {}".format(vec, possiblePosition))
            return

        self.player.move(vec)


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
            Button("Quit", lambda: (logging.info("Bye!"), raise_(StopApplication("Quit"))))
        ]

        for i, button in enumerate(buttons):
            self.add_widget(button, i)

        buttons[active_tab_idx].disabled = True


class SwagLabel(asciimatics.widgets.Widget):
    """
    A text label. But swaggy.
    This class was made to test how to extend Widget class.
    """

    __slots__ = ["_text", "_required_height", "_align"]

    def __init__(self, label, height=1, align="<", name=None):
        """
        :param label: The text to be displayed for the Label.
        :param height: Optional height for the label.  Defaults to 1 line.
        :param align: Optional alignment for the Label.  Defaults to left aligned.
            Options are "<" = left, ">" = right and "^" = centre
        :param name: The name of this widget.

        """
        # Labels have no value and so should have no name for look-ups either.
        super(SwagLabel, self).__init__(name, tab_stop=False)

        # Although this is a label, we don't want it to contribute to the layout
        # tab calculations, so leave internal `_label` value as None.
        # Also ensure that the label really is text.
        self._text = str(label)
        self._required_height = height
        self._align = align

    def process_event(self, event):
        # Labels have no user interactions
        return event

    def update(self, frame_no):
        (colour, attr, background) = self._frame.palette[
            self._pick_palette_key("label", selected=False, allow_input_state=False)]
        swag = "[ {SwagLabel} oh yeahh! ]"
        for i, text in enumerate(
                _split_text(self._text, self._w, self._h, self._frame.canvas.unicode_aware)):
            text = swag + " " + text
            self._frame.canvas.paint(
                "{:{}{}}".format(text, self._align, self._w),
                self._x, self._y + i, colour, attr, background
            )

    def reset(self):
        pass

    def required_height(self, offset, width):
        # Allow one line for text and a blank spacer before it.
        return self._required_height

    @property
    def text(self):
        """
        The current text for this Label.
        """
        return self._text

    @text.setter
    def text(self, new_value):
        self._text = new_value

    @property
    def value(self):
        """
        The current value for this Label.
        """
        return self._value


class inputUtil:

    @staticmethod
    def handle_movement(keyboardEvent: KeyboardEvent) -> Union[None, Tuple[int, int]]:
        """
        :param keyboardEvent:
        :return: Vector the input resolves to.
        """
        inputmap = {
            'w': (0, -1),
            'a': (-1, 0),
            's': (0, 1),
            'd': (1, 0)
        }

        datKey = chr(keyboardEvent.key_code).lower()
        if datKey in inputmap.keys():
            return inputmap[datKey]

        return None


class GameWidget(asciimatics.widgets.Widget):
    __slots__ = ["_required_height", '_game', '_align']

    def __init__(self, game: Game, align='<', name: str = None):
        super(GameWidget, self).__init__(name, tab_stop=False)

        self.game = game
        self._required_height = self.game.world.get_height() + 2
        self._align = align

        self._frame: Frame

    # noinspection PyTypeHints
    def update(self, frame_no: int):
        self._frame.canvas: Canvas

        content = f'|~-~ World {self.game.world.name} ~-~|\n'
        content += f'wew lad (frame_no % 100) = {(frame_no % 100):03d}\n'

        for row in self.game.render_world():
            for char in row:
                content += char
            content += '\n'

        (colour, attr, background) = self._frame.palette[
            self._pick_palette_key("label", selected=False, allow_input_state=False)
        ]

        for i, text in enumerate(_split_text(content, self._w, self._h, self._frame.canvas.unicode_aware)):
            self._frame.canvas.paint(
                f"{text:{self._align}{self._w}}",
                self._x, self._y + i, colour, attr, background
            )

    def reset(self):
        pass

    def process_event(self, event):
        # this widget has no user interactions
        return event

    def required_height(self, offset, width):
        return self._game.world.get_height()

    @property
    def text(self):
        """
        The current text for this Label.
        """
        return self._text

    @text.setter
    def text(self, new_value):
        self._text = new_value

    @property
    def value(self):
        """
        The current value for this Label.
        """
        return self._value

    @property
    def game(self):
        return self._game

    @game.setter
    def game(self, new_value):
        self._game = new_value


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
                                                    label="[focus input/type in me!]", readonly=False)
        layout1.add_widget(self.textTileGen)

        self.widgetGame = GameWidget(
            name="widgetGame",
            game=GAME
        )
        layout1.add_widget(self.widgetGame)

        self.labelFoo = SwagLabel(name="labelFoo", label='foo :)')
        layout1.add_widget(self.labelFoo)

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

    def handle_event(event: Union[KeyboardEvent, MouseEvent]):

        daScene: Scene = screen.current_scene
        daEffects: List[Effect] = daScene.effects

        if len(daEffects) <= 0:
            logging.debug("No effects ;_;")
            return

        maybeDaRootPage: RootPage = daEffects[0]
        maybeDaRootPage.set_theme('bright')  # TODO can we set this earlier, and set it once?

        if not isinstance(event, KeyboardEvent):
            # print("not keyboard event, ignoring... - {}".format(event))
            return
        event: KeyboardEvent

        daChar = chr(event.key_code)
        # pprint(event)

        if maybeDaRootPage.title.strip() == 'Root Page':
            daRootPage = maybeDaRootPage
            # screen.set_title("HOLD UP, YOU PRESSING " + daChar + "?")
            daRootPage.labelFoo.text = f"pressing {daChar}?"

            moveVec = inputUtil.handle_movement(event)
            if moveVec:
                daRootPage.labelFoo.text += ("... you move ({:3d}{:3d})".format(*moveVec))
                GAME.move_player(moveVec)
            else:
                daRootPage.labelFoo.text += "... '{}' is not a movement key.".format(daChar)

        else:
            logging.debug("Not supposed to handle " + maybeDaRootPage.title)

    screen.set_title("dwarfasciigame test :3")
    screen.play(scenes, stop_on_resize=True, start_scene=scene, allow_int=True, unhandled_input=handle_event)


# Press the green button in the gutter to run the script.
if __name__ == '__main__':

    print(f"see {LOGFILENAME} for logs.")

    logging.info('wow its PyCharm!')

    logging.debug(GAME.render_world())

    last_scene = None
    while True:
        try:
            Screen.wrapper(demo, catch_interrupt=True, arguments=[last_scene])
            sys.exit(0)
        except ResizeScreenError as e:
            last_scene = e.scene

# See PyCharm help at https://www.jetbrains.com/help/pycharm/

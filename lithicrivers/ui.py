import logging
from typing import Union, Tuple, List

import asciimatics.widgets
from asciimatics.effects import Effect
from asciimatics.event import KeyboardEvent, MouseEvent
from asciimatics.exceptions import NextScene, StopApplication
from asciimatics.scene import Scene
from asciimatics.screen import Canvas, Screen
from asciimatics.widgets import Layout, Divider, Button, _split_text, Frame, Label
from lithicrivers.settings import GAME_NAME

from lithicrivers.game import Game, Tile, Tiles


class TabButtons(Layout):
    def __init__(self, frame, active_tab_idx):
        cols = [1, 1, 1, 1, 1]

        super().__init__(cols)

        self._frame = frame

        for i, _ in enumerate(cols):
            self.add_widget(Divider(), i)

        buttons = [
            Button("Root Page", lambda: raise_(NextScene("RootPage"))),
            Button("Help Page", lambda: raise_(NextScene("HelpPage"))),
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
                # logging.debug('printchar: {}'.format(char))
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
    __slots__ = ['game']

    def __init__(self, screen, game: Game):
        super().__init__(screen,
                         screen.height,
                         screen.width,
                         can_scroll=False,
                         title="Root Page")

        self.game = game

        layout1 = Layout([1], fill_frame=True)

        self.add_layout(layout1)

        self.labelFeet = asciimatics.widgets.Label(name='labelFeet', label='at your feet rests...[idk bro]')
        layout1.add_widget(self.labelFeet)
        self.labelFeet.text = 'Below your feet is a [{}].'.format(self.game.get_tile_at_player_feet())

        self.widgetGame = GameWidget(
            name="widgetGame",
            game=self.game
        )
        layout1.add_widget(self.widgetGame)

        self.labelFoo = Label(name="labelFoo", label='foo :)')
        layout1.add_widget(self.labelFoo)

        layoutButtons = TabButtons(self, 0)
        self.add_layout(layoutButtons)
        self.fix()


class HelpPage(Frame):
    def __init__(self, screen):
        super().__init__(screen,
                         screen.height,
                         screen.width,
                         can_scroll=False,
                         title="Help Page")
        layout1 = Layout([1], fill_frame=True)
        self.add_layout(layout1)
        # add your widgets here

        layout1.add_widget(Label("keys: WASD, U, arrows, TAB, ENTER...try clicking :P"))

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


class InputHandler:

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

    @staticmethod
    def handle_mining(event: KeyboardEvent, game: Game, root_page: RootPage):
        # TODO: clean up state... :P why do we pass all these as args?
        char = chr(event.key_code)

        if not (char.lower() == 'u'):
            return

        root_page.labelFoo.text += "...but it is a mining key!"

        tile_under: Tile = game.get_tile_at_player_feet()
        if tile_under == Tiles.Dirt():
            root_page.labelFoo.text += ' You can\'t mine dirt :P'
            return  # can't mine dirt

        if tile_under == Tiles.DaFuq():
            root_page.labelFoo.text += ' What da fuq is that tile? :3c'


def demo(screen: Screen, scene: Scene, game: Game):
    scenes = [
        Scene([RootPage(screen, game)], -1, name="RootPage"),
        Scene([HelpPage(screen)], -1, name="HelpPage"),
        Scene([BravoPage(screen)], -1, name="BravoPage"),
        Scene([CharliePage(screen)], -1, name="CharliePage"),
    ]

    def handle_event(event: Union[KeyboardEvent, MouseEvent]):

        daScene: Scene = screen.current_scene
        daEffects: List[Effect] = daScene.effects

        if len(daEffects) <= 0:
            logging.debug("No effects ;_;")
            return

        maybe_root_page: RootPage = daEffects[0]
        maybe_root_page.set_theme('bright')  # TODO can we set this earlier, and set it once?

        if not isinstance(event, KeyboardEvent):
            # print("not keyboard event, ignoring... - {}".format(event))
            return
        event: KeyboardEvent

        char = chr(event.key_code)
        # pprint(event)

        if maybe_root_page.title.strip() == 'Root Page':
            root_page = maybe_root_page
            # screen.set_title("HOLD UP, YOU PRESSING " + char + "?")
            root_page.labelFoo.text = f"pressing {char}?"

            moveVec = InputHandler.handle_movement(event)
            if moveVec:
                root_page.labelFoo.text += ("... you move ({:3d}{:3d})".format(*moveVec))
                root_page.game.move_player(moveVec)

                root_page.labelFeet.text = 'Below your feet is a [{}].'.format(root_page.game.get_tile_at_player_feet())

            else:
                root_page.labelFoo.text += "... '{}' is not a movement key.".format(char)

            InputHandler.handle_mining(event, root_page.game, root_page)

        else:
            logging.debug("Not supposed to handle " + maybe_root_page.title)

    screen.set_title("~~-[ {} ]-~~".format(GAME_NAME))
    screen.play(scenes, stop_on_resize=True, start_scene=scene, allow_int=True, unhandled_input=handle_event)


def raise_(ex):
    """Because we can't use raise in lambda for some reason..."""
    raise ex

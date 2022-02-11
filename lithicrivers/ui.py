import logging
from typing import Union, Tuple, List

import asciimatics.widgets
from asciimatics.effects import Effect
from asciimatics.event import KeyboardEvent, MouseEvent
from asciimatics.exceptions import NextScene, StopApplication
from asciimatics.scene import Scene
from asciimatics.screen import Canvas, Screen
from asciimatics.widgets import Layout, Divider, Button, _split_text, Frame, Label

from lithicrivers.game import Game, Tile, Tiles
from lithicrivers.model import Vector2
from lithicrivers.settings import GAME_NAME, KEYMAP


class TabButtons(Layout):
    def __init__(self, frame, active_tab_idx):
        cols = [1, 1, 1, 1, 1]

        super().__init__(cols)

        self._frame = frame

        for i, _ in enumerate(cols):
            self.add_widget(Divider(), i)

        buttons = [
            Button("Root Page", raiseFn(NextScene, "RootPage")),
            Button("Help Page", raiseFn(NextScene, "HelpPage")),
            Button("Bravo Page", raiseFn(NextScene, "BravoPage")),
            Button("Charlie Page", raiseFn(NextScene, "CharliePage")),
            Button("Quit", lambda: (logging.info("Bye!"), raise_(StopApplication("Quit"))))
        ]

        for i, button in enumerate(buttons):
            self.add_widget(button, i)

        buttons[active_tab_idx].disabled = True


class GameWidget(asciimatics.widgets.Widget):
    __slots__ = ['_game', '_align']

    def __init__(self, game: Game, align='<', name: str = None):
        super(GameWidget, self).__init__(name, tab_stop=False)

        self.game = game
        self._align = align

        self._frame: Frame

        # print("we need {} height...".format(self.required_height(0, 0)))
        # print(self.game.viewport)

    def required_height(self, offset, width):
        return self.game.viewport.get_height() + 2  # +2 for our random text shit

    # noinspection PyTypeHints
    def update(self, frame_no: int):
        self._frame.canvas: Canvas

        content = f'|~-~ World {self.game.world.name} ~-~|\n'
        content += f'wew lad (frame_no % 100) = {(frame_no % 100):03d}\n'

        toRender = self.game.render_world_viewport()

        logging.debug("CONTENT RENDERED:")
        logging.debug(content)

        for row in toRender:
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

        self.labelFoo = Label(name="labelFoo", label='foo :)', height=1)
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
    def handle_movement(keyboardEvent: KeyboardEvent) -> Union[None, Vector2]:
        """
        :param keyboardEvent:
        :return: Vector the input resolves to.
        """

        datKey = chr(keyboardEvent.key_code).lower()
        if datKey in KEYMAP.MOVEMENT_VECTOR_MAP.keys():
            return KEYMAP.MOVEMENT_VECTOR_MAP[datKey]

        return None

    @staticmethod
    def handle_mining(event: KeyboardEvent, game: Game, root_page: RootPage):
        # TODO: clean up state... :P why do we pass all these as args?

        if not KEYMAP.matches_keyboard_event('MINE', event):
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
            root_page.labelFoo.text = f"pressing {char}? "

            moveVec = InputHandler.handle_movement(event)
            if moveVec:
                root_page.game.move_player(moveVec)

                root_page.labelFoo.text += ("pos={:02d},{:02d}".format(game.player.position.x, game.player.position.y))

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


def raiseFn(clazz: any, name: str):
    """
    bruh, why?
    """

    def raiseNextScene(arg=name):
        raise clazz(arg)

    return raiseNextScene

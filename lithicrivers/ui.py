import logging
from typing import Union, List

import asciimatics.widgets
from asciimatics.effects import Effect
from asciimatics.event import KeyboardEvent, MouseEvent
from asciimatics.exceptions import NextScene
from asciimatics.scene import Scene
from asciimatics.screen import Canvas, Screen
from asciimatics.widgets import Layout, Divider, Button, _split_text, Frame, Label

from lithicrivers.game import Game, Tile, Tiles
from lithicrivers.model.modelpleasemoveme import StopGame, RenderedData
from lithicrivers.model.vector import VectorN
from lithicrivers.settings import GAME_NAME, KEYMAP, Keymap, VIEWPORT_WIGGLE
from lithicrivers.textutil import presenting, list_label


# class MainGameFrame(Layout):
#     def __init__(self, frame, active_tab_idx, game: Game = None):
#         cols = [1]
#
#         raise NotImplementedError("lazy!")


class TabButtons(Layout):

    def __init__(self, frame, active_tab_idx, game: Game = None):
        cols = [1, 1, 1, 1, 1]

        super().__init__(cols)

        self._frame = frame
        self.game = game

        for i, _ in enumerate(cols):
            self.add_widget(Divider(), i)

        buttons = [
            Button("Help", raiseFn(NextScene, "HelpPage")),
            Button("Root Page", raiseFn(NextScene, "RootPage")),
            Button("Message Log", raiseFn(NextScene, "MessageLogPage")),
            Button("Charlie Page", raiseFn(NextScene, "CharliePage")),
            Button("Quit", raiseFn(StopGame, "Game stopping :P"))
        ]

        for i, button in enumerate(buttons):
            self.add_widget(button, i)

        buttons[active_tab_idx].disabled = True


class HeaderLabel(asciimatics.widgets.Widget):
    """
    A text label. But with a header.
    This class was originally made to test how to extend Widget class.
    """

    __slots__ = ["_text", "_required_height", "_align", 'header']

    def __init__(self, label='', height=1, align="<", name=None, header='???'):
        """
        :param label: The text to be displayed for the Label.
        :param height: Optional height for the label.  Defaults to 1 line.
        :param align: Optional alignment for the Label.  Defaults to left aligned.
            Options are "<" = left, ">" = right and "^" = centre
        :param name: The name of this widget.

        """
        # Labels have no value and so should have no name for look-ups either.
        super(HeaderLabel, self).__init__(name, tab_stop=False)

        # Although this is a label, we don't want it to contribute to the layout
        # tab calculations, so leave internal `_label` value as None.
        # Also ensure that the label really is text.
        self._text = str(label)
        self._required_height = height
        self._align = align
        self.header = header

    def process_event(self, event):
        # Labels have no user interactions
        return event

    def update(self, frame_no):
        (colour, attr, background) = self._frame.palette[
            self._pick_palette_key("label", selected=False, allow_input_state=False)]
        headerPrefix = list_label(self.header)
        for i, text in enumerate(
                _split_text(self._text, self._w, self._h, self._frame.canvas.unicode_aware)):
            text = headerPrefix + " " + text
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

        content = ""
        content += f'|~-~ World {self.game.world.name} ~-~|\n'

        toRender: RenderedData = self.game.render_world_viewport()

        content += toRender.as_string()

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
                         can_scroll=True,
                         title="Root Page")

        columns = [70, 30]

        self.game = game

        layout1 = Layout(columns=columns, fill_frame=True)

        self.add_layout(layout1)

        self.labelMessage = HeaderLabel(name='labelMessage', header='MSG')
        layout1.add_widget(self.labelMessage, column=1)
        self.labelMessage.text = "Welcome to {}! <3".format(game.world.name)

        self.labelPosition = HeaderLabel(name='labelPosition', header="POS")
        layout1.add_widget(self.labelPosition, column=1)
        self.labelPosition.text = str(self.game.render_pretty_player_position())

        self.labelViewport = HeaderLabel(name='labelViewport', header="VIEW")
        layout1.add_widget(self.labelViewport, column=1)
        self.labelViewport.text = str(self.game.viewport.render_pretty())

        self.labelFeet = HeaderLabel(name='labelFeet', header="FEET")
        layout1.add_widget(self.labelFeet, column=1)
        self.labelFeet.text = str(self.game.get_tile_at_player_feet())

        self.labelInventory = HeaderLabel(name='labelInventory', header='INV')
        layout1.add_widget(self.labelInventory, column=1)
        self.labelInventory.text = self.game.player.inventory.summary()

        self.widgetGame = GameWidget(
            name="widgetGame",
            game=self.game
        )
        layout1.add_widget(self.widgetGame, column=0)

        layoutButtons = TabButtons(self, 1)
        self.add_layout(layoutButtons)
        self.fix()


class HelpPage(Frame):
    def __init__(self, screen, game: Game):
        super().__init__(screen,
                         screen.height,
                         screen.width,
                         can_scroll=False,
                         title="Help")
        layout1 = Layout([1], fill_frame=True)
        self.add_layout(layout1)
        # add your widgets here

        helptxt = ""
        helptxt = (f"Hello! Welcome to {GAME_NAME}. Below are keys.\n"
                   "By the way, game UI nav is arrow keys + space or enter.\n"
                   "You can also use the mouse! Left click works!\n"
                   "Enjoy!\n"
                   "\n"
                   f"Your character's appearance: {presenting(game.player.render_sprite(1))}\n")

        helptxt += KEYMAP.generate_key_guide()

        helptxtheight = len(helptxt.split('\n'))

        helpLabel = Label(helptxt, height=helptxtheight)

        layout1.add_widget(helpLabel)

        layout2 = TabButtons(self, 0)
        self.add_layout(layout2)
        self.fix()


class MessageLogPage(Frame):
    def __init__(self, screen):
        super().__init__(screen,
                         screen.height,
                         screen.width,
                         can_scroll=False,
                         title="Message Log")
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
    def handle_movement(keyboardEvent: KeyboardEvent) -> Union[None, VectorN]:
        """
        :param keyboardEvent:
        :return: Vector the input resolves to.
        """

        datKey = Keymap.char_from_keyboard_event(keyboardEvent)
        if datKey in KEYMAP.MOVEMENT_VECTOR_MAP.keys():
            return KEYMAP.MOVEMENT_VECTOR_MAP[datKey]

        return None

    @staticmethod
    def handle_mining(event: KeyboardEvent, game: Game, root_page: RootPage):
        # TODO: clean up state... :P why do we pass all these as args?

        if not KEYMAP.matches('MINE', event):
            return

        tile_under: Tile = game.get_tile_at_player_feet()
        if tile_under == Tiles.Dirt():
            root_page.labelMessage.text = 'You can\'t mine dirt :P'
            return  # can't mine dirt
        elif tile_under == Tiles.Tree():
            root_page.labelMessage.text = 'Yell at me to let you mine trees. I\'m lazy.'
            return  # can't mine dirt
        elif tile_under == Tiles.DaFuq():
            game.player.inventory.add_item(tile_under.calc_drop())
            game.set_tile_at_player_feet(Tiles.Dirt())

    @classmethod
    def handle_viewport(cls, event: KeyboardEvent, game: Game):

        if KEYMAP.matches('RESET_VIEWPORT', event):
            game.reset_viewport()

        if KEYMAP.matches('SLIDE_VIEWPORT_WEST', event):
            game.viewport.slide_left()

        if KEYMAP.matches('SLIDE_VIEWPORT_EAST', event):
            game.viewport.slide_right()

    @classmethod
    def handle_scale(cls, event, game):
        if KEYMAP.matches('SCALE_DOWN', event):
            game.viewport.rescale_down(1)
            game.reset_viewport()

        if KEYMAP.matches('SCALE_UP', event):
            game.viewport.rescale_up(1)
            game.reset_viewport()


def demo(screen: Screen, scene: Scene, game: Game):
    scenes = [
        Scene([HelpPage(screen, game)], -1, name="HelpPage"),
        Scene([RootPage(screen, game)], -1, name="RootPage"),
        Scene([MessageLogPage(screen)], -1, name="MessageLogPage"),
        Scene([CharliePage(screen)], -1, name="CharliePage"),
    ]

    for scene in scenes[::-1]:
        scene.effects[0].set_theme('bright')

    def handle_event(event: Union[KeyboardEvent, MouseEvent]):

        daScene: Scene = screen.current_scene
        daEffects: List[Effect] = daScene.effects

        if len(daEffects) <= 0:
            logging.debug("No effects ;_;")
            return

        maybe_root_page: RootPage = daEffects[0]

        if not isinstance(event, KeyboardEvent):
            # print("not keyboard event, ignoring... - {}".format(event))
            return
        event: KeyboardEvent

        if maybe_root_page.title.strip() != 'Root Page':
            logging.debug("Not supposed to handle " + maybe_root_page.title)
            return

        root_page = maybe_root_page

        move_vec = InputHandler.handle_movement(event)
        if move_vec:

            root_page.game.move_player(move_vec)

            # display pos
            root_page.labelPosition.text = game.render_pretty_player_position()

            # move the viewport with the player
            if game.player_outside_viewport(wiggle=VIEWPORT_WIGGLE):
                game.viewport.slide(move_vec)

                # still outside? Something's wrong, let's reset the viewport...
                if game.player_outside_viewport(wiggle=VIEWPORT_WIGGLE):
                    game.reset_viewport()

        InputHandler.handle_viewport(event, root_page.game)
        InputHandler.handle_scale(event, root_page.game)

        InputHandler.handle_mining(event, root_page.game, root_page)
        root_page.labelInventory.text = root_page.game.player.inventory.summary()

        # after we mine
        root_page.labelFeet.text = str(root_page.game.get_tile_at_player_feet())

        # after we handle viewport

        root_page.labelViewport.text = str(root_page.game.viewport.render_pretty())

    screen.set_title("~~-[ {} ]-~~".format(GAME_NAME))
    screen.play(scenes, stop_on_resize=True, start_scene=scene, allow_int=True, unhandled_input=handle_event)


def _raise(ex):
    """Because we can't use raise in lambda for some reason..."""
    raise ex


def raiseFn(clazz: any, name: str):
    """
    bruh, why?
    """

    def raiseNextScene(arg=name):
        raise clazz(arg)

    return raiseNextScene

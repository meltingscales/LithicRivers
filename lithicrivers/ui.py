import logging
from typing import Union, List, Tuple

import asciimatics.widgets
from asciimatics.effects import Effect
from asciimatics.event import KeyboardEvent, MouseEvent
from asciimatics.exceptions import NextScene
from asciimatics.scene import Scene
from asciimatics.screen import Canvas, Screen
from asciimatics.widgets import Layout, Divider, Button, _split_text, Frame, Label

from lithicrivers.game import Game, Tiles, Items
from lithicrivers.model.vector import VectorN
from lithicrivers.model.modelpleasemoveme import RenderedData, StopGame
from lithicrivers.textutil import get_color_for_ui_element, presenting, list_label
from lithicrivers.keymap import KEYMAP
from lithicrivers.settings import GAME_NAME, VIEWPORT_WIGGLE


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
            Button("Extra Page", raiseFn(NextScene, "ExtraPage")),
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
        self._frame.canvas: Canvas

        headerPrefix = list_label(self.header)
        
        # Get colors for header and content
        header_color = get_color_for_ui_element("TITLE")
        
        # Determine content color based on message type
        if self._text.startswith('[ERROR]'):
            content_color = get_color_for_ui_element("ERROR")
        elif self._text.startswith('[SUCCESS]'):
            content_color = get_color_for_ui_element("SUCCESS")
        elif self._text.startswith('[WARNING]'):
            content_color = get_color_for_ui_element("WARNING")
        elif self._text.startswith('[INFO]'):
            content_color = get_color_for_ui_element("INFO")
        elif self._text.startswith('[RARE]'):
            content_color = get_color_for_ui_element("RARE")
        elif self._text.startswith('[VALUABLE]'):
            content_color = get_color_for_ui_element("VALUABLE")
        else:
            content_color = get_color_for_ui_element("LABEL")
        
        # Render header with title color
        self._frame.canvas.paint(
            headerPrefix,
            self._x, self._y, header_color[0], header_color[1], header_color[2]
        )
        
        # Render content with appropriate color
        self._frame.canvas.paint(
            self._text,
            self._x + len(headerPrefix), self._y, content_color[0], content_color[1], content_color[2]
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

        # Render the world with colors
        self._render_colored_world(toRender)
        
        # Render the header
        header_color = get_color_for_ui_element("HEADER")
        self._frame.canvas.paint(
            f"{content:{self._align}{self._w}}",
            self._x, self._y, header_color[0], header_color[1], header_color[2]
        )

    def _render_colored_world(self, rendered_data: RenderedData):
        """Render the world with proper colors."""
        start_y = self._y + 1  # Start after the header
        
        for y in range(len(rendered_data.render_data)):
            render_row = rendered_data.render_data[y]
            for stripe_idx in range(rendered_data.scale):
                row_content = ""
                row_colors = []
                
                for x in range(len(render_row)):
                    render_item = render_row[x]
                    render_item_chunk = render_item.split('\n')
                    slice = render_item_chunk[stripe_idx] if stripe_idx < len(render_item_chunk) else " "
                    slice = slice.replace('\n', '')
                    row_content += slice
                    
                    # Get color for this position and repeat it for each character in the scaled sprite
                    tile_color = rendered_data.get_color_at(x, y)
                    # Repeat the color for each character in the scaled sprite slice
                    for _ in range(len(slice)):
                        row_colors.append(tile_color)
                
                # Render this row with colors
                self._render_colored_row(row_content, row_colors, start_y + y * rendered_data.scale + stripe_idx)
    
    def _render_colored_row(self, content: str, colors: List[Tuple[int, int, int]], y_pos: int):
        """Render a row with individual character colors."""
        x_pos = self._x
        
        for i, char in enumerate(content):
            if i < len(colors):
                color = colors[i]
            else:
                color = get_color_for_ui_element("DEFAULT")
            
            # Paint each character with its color
            self._frame.canvas.paint(
                char,
                x_pos + i, y_pos, color[0], color[1], color[2]
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
        self.labelInventory.text = self.game.player.inventory.colored_summary()

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

        helpLabel = Label(helptxt, height=helptxtheight, name="helpLabel")

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


class ExtraPage(Frame):
    def __init__(self, screen):
        super().__init__(screen,
                         screen.height,
                         screen.width,
                         can_scroll=False,
                         title="Extra Page")
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

        # First check for numpad movement
        if KEYMAP.matches_numpad(keyboardEvent):
            return KEYMAP.get_numpad_movement_vector(keyboardEvent)

        # Then check for regular character movement
        datKey = KEYMAP.char_from_keyboard_event(keyboardEvent)
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
            root_page.labelMessage.text = '[ERROR] You can\'t mine dirt :P'
            return  # can't mine dirt
        elif tile_under == Tiles.Tree():
            # Allow mining trees - they drop guaranteed acorns plus other items
            tree_drops = tile_under.calc_tree_drops()
            for item in tree_drops:
                game.player.inventory.add_item(item)
            game.set_tile_at_player_feet(Tiles.Dirt())
            root_page.labelMessage.text = '[SUCCESS] You chopped down the tree!'
        elif tile_under == Tiles.Gold_Ore():
            game.player.inventory.add_item(tile_under.calc_drop())
            game.set_tile_at_player_feet(Tiles.Dirt())
            root_page.labelMessage.text = '[RARE] You found something mysterious!'

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
        Scene([ExtraPage(screen)], -1, name="ExtraPage"),
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

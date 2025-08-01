import logging
from typing import Union, List, Tuple, Optional, Callable

import asciimatics.widgets
from asciimatics.effects import Effect
from asciimatics.event import KeyboardEvent, MouseEvent
from asciimatics.exceptions import NextScene
from asciimatics.scene import Scene
from asciimatics.screen import Canvas, Screen
from asciimatics.widgets import Layout, Divider, Button, _split_text, Frame, Label, TextBox, RadioButtons

from lithicrivers.game import Game, Tiles, Items, NPC
from lithicrivers.model.vector import VectorN
from lithicrivers.model.modelpleasemoveme import RenderedData, StopGame
from lithicrivers.textutil import get_color_for_ui_element, presenting, list_label
from lithicrivers.keymap import KEYMAP
from lithicrivers.settings import GAME_NAME, VIEWPORT_WIGGLE
from lithicrivers.model.modelpleasemoveme import Viewport


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
            Button("Test Popups", raiseFn(NextScene, "ExtraPage")),
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
        # Account for scale: each tile takes up scale characters vertically
        return self.game.viewport.get_height() * self.game.viewport.scale + 2  # +2 for our random text shit
    
    def required_width(self, offset, width):
        # Account for scale: each tile takes up scale characters horizontally
        return self.game.viewport.get_width() * self.game.viewport.scale

    # noinspection PyTypeHints
    def update(self, frame_no: int):
        self._frame.canvas: Canvas

        content = ""
        content += f'|~-~ World {self.game.world.name} ~-~|\n'

        # Check if viewport should be visible
        viewport_visible = getattr(self.game, 'viewport_visible', True)
        
        if viewport_visible:
            toRender: RenderedData = self.game.render_world_viewport()
            # Render the world with colors
            self._render_colored_world(toRender)
        # If viewport is hidden, render nothing at all
        
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
        
        # Clamp content to available width to prevent overflow
        max_width = self._w if hasattr(self, '_w') else len(content)
        clamped_content = content[:max_width]
        clamped_colors = colors[:max_width]
        
        for i, char in enumerate(clamped_content):
            if i < len(clamped_colors):
                color = clamped_colors[i]
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

        # Use more flexible column layout - game widget gets more space
        # Calculate columns based on screen width for better space utilization
        screen_width = screen.width
        if screen_width >= 120:
            # Large screen: give more space to game widget
            columns = [80, 20]
        elif screen_width >= 80:
            # Medium screen: balanced layout
            columns = [75, 25]
        else:
            # Small screen: minimal info panel
            columns = [70, 30]

        self.game = game
        
        # Adjust viewport size based on available screen space
        self._adjust_viewport_for_screen(screen)

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

    def _adjust_viewport_for_screen(self, screen):
        """Adjust viewport size based on available screen space."""
        # Calculate available space for the game widget
        screen_width = screen.width
        screen_height = screen.height
        
        # Account for info panel width (20-30% depending on screen size)
        if screen_width >= 120:
            info_panel_width = int(screen_width * 0.20)
        elif screen_width >= 80:
            info_panel_width = int(screen_width * 0.25)
        else:
            info_panel_width = int(screen_width * 0.30)
        
        # Account for borders, headers, and tab buttons
        available_width = screen_width - info_panel_width - 4  # 4 for borders
        available_height = screen_height - 6  # 6 for headers, borders, and tab buttons
        
        # Calculate optimal viewport size
        # Each tile takes up scale characters, so we need to account for that
        scale = self.game.viewport.scale
        max_tiles_x = available_width // scale
        max_tiles_y = available_height // scale
        
        # Ensure we have at least a minimum viewport size
        min_tiles = 5
        max_tiles_x = max(max_tiles_x, min_tiles)
        max_tiles_y = max(max_tiles_y, min_tiles)
        
        # Calculate radius (half the viewport size)
        radius_x = max_tiles_x // 2
        radius_y = max_tiles_y // 2
        
        # Ensure radius is at least 1
        radius_x = max(radius_x, 1)
        radius_y = max(radius_y, 1)
        
        # Update viewport with new radius
        new_viewport = Viewport.generate_centered(
            self.game.player.position,
            radius=VectorN(radius_x, radius_y, 0),
            scale=self.game.viewport.scale
        )
        self.game.viewport = new_viewport

    def handle_terminal_resize(self, screen):
        """Handle terminal resize by recalculating viewport size."""
        self._adjust_viewport_for_screen(screen)
        # Update viewport display
        self.labelViewport.text = str(self.game.viewport.render_pretty())


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
                   f"Your character's appearance: {presenting(game.player.render_sprite(1))}\n"
                   "\n"
                   "=== KEYBINDS ===\n")

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
                         title="Test Popups")
        layout1 = Layout([1], fill_frame=True)
        self.add_layout(layout1)
        
        # Add test popup buttons
        from asciimatics.widgets import Button, Label
        
        def test_simple_dialog():
            """Test a simple dialog without options."""
            def callback(result):
                print(f"Simple dialog result: {result}")
            
            # Use asciimatics PopUpDialog for simple dialog
            from asciimatics.widgets import PopUpDialog
            popup = PopUpDialog(
                screen,
                "This is a test dialog with no options.\nPress OK to continue.",
                ["OK"],
                callback
            )
            # Add the popup to the current scene
            screen.current_scene.add_effect(popup)
        
        def test_options_dialog():
            """Test a dialog with options."""
            def callback(result):
                print(f"Options dialog result: {result}")
            
            # Use asciimatics PopUpDialog for options dialog
            from asciimatics.widgets import PopUpDialog
            popup = PopUpDialog(
                screen,
                "This is a test dialog with options.\nSelect an option:",
                ["Option 1", "Option 2", "Option 3", "Option 4"],
                callback
            )
            # Add the popup to the current scene
            screen.current_scene.add_effect(popup)
        
        def test_large_dialog():
            """Test a large dialog with lots of content."""
            def callback(result):
                print(f"Large dialog result: {result}")
            
            # Use asciimatics PopUpDialog for large dialog
            from asciimatics.widgets import PopUpDialog
            popup = PopUpDialog(
                screen,
                "This is a large test dialog with lots of content.\n\n"
                "It has multiple lines of text to test how the dialog handles "
                "long content and multiple paragraphs.\n\n"
                "The dialog should automatically size itself to fit the content "
                "while staying within the screen bounds.",
                ["Continue", "Cancel"],
                callback
            )
            # Add the popup to the current scene
            screen.current_scene.add_effect(popup)
        
        def test_popup_box():
            """Test asciimatics PopUpDialog."""
            from asciimatics.widgets import PopUpDialog
            
            def callback(result):
                print(f"Popup dialog result: {result}")
            
            # Create a popup dialog using asciimatics PopUpDialog
            popup = PopUpDialog(
                screen,
                "This is a test popup dialog.\n\n"
                "This uses the built-in asciimatics PopupDialog widget.\n"
                "It should work much better than our custom implementation.",
                ["OK", "Cancel"],
                callback
            )
            # Add the popup to the current scene
            screen.current_scene.add_effect(popup)
        
        # Add buttons to test different dialog types
        layout1.add_widget(Button("Test Simple Dialog", test_simple_dialog))
        layout1.add_widget(Button("Test Options Dialog", test_options_dialog))
        layout1.add_widget(Button("Test Large Dialog", test_large_dialog))
        layout1.add_widget(Button("Test Popup Dialog", test_popup_box))
        
        # Add info text
        info_label = Label("Click the buttons above to test different dialog types.\n"
                          "These use proper asciimatics components instead of custom widgets.\n"
                          "Press 'v' in the main game to toggle viewport visibility.")
        layout1.add_widget(info_label)

        layout2 = TabButtons(self, 3)
        self.add_layout(layout2)
        self.fix()


class DialogBox(Frame):
    """A modal dialog box for conversations and interactions."""
    
    def __init__(self, screen, title: str, content: str, options: List[str] = None, 
                 callback: Optional[Callable] = None, game: Game = None):
        # Calculate dialog size based on content
        max_width = min(80, screen.width - 4)
        max_height = min(20, screen.height - 4)
        
        # Calculate required height for content
        lines = _split_text(content, max_width - 4, max_height)
        content_height = len(lines)
        
        # Add height for options if present
        if options:
            content_height += len(options) + 2
        
        # Ensure minimum height
        height = max(content_height + 4, 8)
        
        super().__init__(
            screen,
            height,
            max_width,
            title=title,
            can_scroll=False,
            has_border=True
        )
        
        self.game = game
        self.callback = callback
        self.options = options
        
        # Create layout
        layout = Layout([1], fill_frame=True)
        self.add_layout(layout)
        
        # Add content
        content_widget = TextBox(content_height, content, name="content", readonly=True)
        layout.add_widget(content_widget)
        
        # Add options if present
        if options:
            self.options_widget = RadioButtons(
                options,
                label="Choose an option:",
                name="options"
            )
            layout.add_widget(self.options_widget)
        
        # Add buttons
        button_layout = Layout([1, 1])
        self.add_layout(button_layout)
        
        if options:
            button_layout.add_widget(Button("Select", self._on_select), 0)
        else:
            button_layout.add_widget(Button("OK", self._on_ok), 0)
        
        button_layout.add_widget(Button("Cancel", self._on_cancel), 1)
    
    def _on_select(self):
        """Handle option selection."""
        if self.options_widget and self.callback:
            selected = self.options_widget.value
            if 0 <= selected < len(self.options):
                self.callback(self.options[selected])
        self._close()
    
    def _on_ok(self):
        """Handle OK button."""
        if self.callback:
            self.callback(None)
        self._close()
    
    def _on_cancel(self):
        """Handle Cancel button."""
        self._close()
    
    def _close(self):
        """Close the dialog."""
        # Remove this dialog from the current scene
        if hasattr(self, '_screen') and self._screen.current_scene:
            self._screen.current_scene.remove_effect(self)


class EntitySelectionPopup(Frame):
    """A modal popup for selecting which entity to interact with."""
    
    def __init__(self, screen, game: Game, adjacent_entities: List[Tuple[str, VectorN, str]]):
        # Calculate popup size and position
        max_width = min(60, screen.width - 4)
        height = len(adjacent_entities) + 8  # +8 for header, buttons, borders, etc.
        
        # Center the popup on screen
        x = (screen.width - max_width) // 2
        y = (screen.height - height) // 2
        
        super().__init__(
            screen,
            height,
            max_width,
            title="Choose Entity to Interact With",
            can_scroll=False,
            has_border=True
        )
        
        self.game = game
        self.adjacent_entities = adjacent_entities
        self.screen = screen
        
        # Create layout
        layout = Layout([1], fill_frame=True)
        self.add_layout(layout)
        
        # Add header
        header = Label(f"Found {len(adjacent_entities)} entities nearby:", name="header")
        layout.add_widget(header)
        
        # Add entity options
        self.entity_widget = RadioButtons(
            [f"{name} ({color})" for name, pos, color in adjacent_entities],
            label="Select an entity:",
            name="entities"
        )
        layout.add_widget(self.entity_widget)
        
        # Add buttons
        button_layout = Layout([1, 1])
        self.add_layout(button_layout)
        
        button_layout.add_widget(Button("Interact", self._on_interact), 0)
        button_layout.add_widget(Button("Cancel", self._on_cancel), 1)
    
    def _on_interact(self):
        """Handle entity selection and interaction."""
        selected = self.entity_widget.value
        if 0 <= selected < len(self.adjacent_entities):
            name, pos, color = self.adjacent_entities[selected]
            self._handle_interaction(name, pos, color)
        self._close()
    
    def _on_cancel(self):
        """Handle cancel."""
        self._close()
    
    def _handle_interaction(self, name: str, pos: VectorN, color: str):
        """Handle the actual interaction."""
        # Get the entity
        entity = self.game.world.get_entity(pos)
        
        if hasattr(entity, 'interact'):
            # For interactive entities, show their interaction text
            interaction_text = entity.interact()
            self._show_interaction_result(name, interaction_text)
        elif hasattr(entity, 'get_conversation'):
            # For NPCs, start conversation
            self._start_npc_conversation(entity)
        else:
            # Default interaction
            self._show_interaction_result(name, f"You interact with {name}.")
    
    def _show_interaction_result(self, name: str, text: str):
        """Show the result of an interaction."""
        # Create a result popup
        result_popup = InteractionResultPopup(
            self.screen,
            f"Interacting with {name}",
            text
        )
        # For now, just show the result in the message area
        # TODO: Implement proper result popup display
    
    def _start_npc_conversation(self, npc):
        """Start a conversation with an NPC."""
        conversation = npc.get_conversation()
        # For now, just show the conversation in the message area
        # TODO: Implement proper conversation dialog
        pass
    
    def _close(self):
        """Close the popup and return to the game."""
        # Remove this popup from the current scene
        if hasattr(self, '_screen') and self._screen.current_scene:
            self._screen.current_scene.remove_effect(self)


class InteractionResultPopup(Frame):
    """A popup to show the result of an interaction."""
    
    def __init__(self, screen, title: str, content: str):
        # Calculate popup size
        max_width = min(70, screen.width - 4)
        lines = _split_text(content, max_width - 4, 10)
        height = len(lines) + 6  # +6 for title, buttons, borders
        
        super().__init__(
            screen,
            height,
            max_width,
            title=title,
            can_scroll=False,
            has_border=True
        )
        
        # Create layout
        layout = Layout([1], fill_frame=True)
        self.add_layout(layout)
        
        # Add content
        content_widget = TextBox(len(lines), content, name="content", readonly=True)
        layout.add_widget(content_widget)
        
        # Add buttons
        button_layout = Layout([1])
        self.add_layout(button_layout)
        
        button_layout.add_widget(Button("OK", self._on_ok), 0)
    
    def _on_ok(self):
        """Handle OK button."""
        self._close()
    
    def _close(self):
        """Close the popup."""
        # Remove this popup from the current scene
        if hasattr(self, '_screen') and self._screen.current_scene:
            self._screen.current_scene.remove_effect(self)





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
            
        if KEYMAP.matches('TOGGLE_VIEWPORT', event):
            # Toggle viewport visibility by setting a flag
            if not hasattr(game, 'viewport_visible'):
                game.viewport_visible = True
            game.viewport_visible = not game.viewport_visible

    @classmethod
    def handle_scale(cls, event, game):
        if KEYMAP.matches('SCALE_DOWN', event):
            game.viewport.rescale_down(1)
            game.reset_viewport()

        if KEYMAP.matches('SCALE_UP', event):
            game.viewport.rescale_up(1)
            game.reset_viewport()

    @classmethod
    def handle_interaction(cls, event: KeyboardEvent, game: Game, root_page: RootPage):
        """Handle interaction with adjacent entities."""
        if not KEYMAP.matches('INTERACT', event):
            return
        
        # Get adjacent entities
        adjacent_entities = game.world.get_adjacent_entities(game.player.position)
        
        if not adjacent_entities:
            root_page.labelMessage.text = '[INFO] Nothing to interact with nearby.'
            return
        
        # Check if there's an NPC adjacent (for conversation)
        for name, pos, color in adjacent_entities:
            entity = game.world.get_entity(pos)
            if isinstance(entity, NPC):
                # Start conversation with NPC
                cls._start_npc_conversation(game, entity, root_page)
                return
        
        # Show interaction popup for other entities
        cls._show_interaction_popup(game, adjacent_entities, root_page)
    
    @classmethod
    def _start_npc_conversation(cls, game: Game, npc: NPC, root_page: RootPage):
        """Start a conversation with an NPC."""
        conversation = npc.get_conversation()
        
        def conversation_callback(selected_option):
            if selected_option:
                # Handle the selected option
                response = selected_option
                next_topic = npc.handle_response(response, "greeting")
                if next_topic:
                    next_conversation = npc.get_conversation(next_topic)
                    cls._show_interaction_result(npc.name, next_conversation["text"], root_page)
        
        # Show the conversation in a popup
        from asciimatics.widgets import PopUpDialog
        popup = PopUpDialog(
            root_page._screen,
            conversation["text"],
            conversation["options"],
            conversation_callback
        )
        # Add the popup to the current scene
        root_page._screen.current_scene.add_effect(popup)
    
    @classmethod
    def _continue_npc_conversation(cls, game: Game, npc: NPC, conversation: dict, root_page: RootPage):
        """Continue an NPC conversation."""
        def conversation_callback(selected_option):
            if selected_option:
                # Handle the selected option
                response = selected_option
                next_topic = npc.handle_response(response, "greeting")
                if next_topic:
                    next_conversation = npc.get_conversation(next_topic)
                    cls._show_interaction_result(npc.name, next_conversation["text"], root_page)
        
        # Show the conversation in a popup
        from asciimatics.widgets import PopUpDialog
        popup = PopUpDialog(
            root_page._screen,
            conversation["text"],
            conversation["options"],
            conversation_callback
        )
        # Add the popup to the current scene
        root_page._screen.current_scene.add_effect(popup)
    
    @classmethod
    def _show_interaction_popup(cls, game: Game, adjacent_entities: List[Tuple[str, VectorN, str]], root_page: RootPage):
        """Show interaction popup for entities."""
        # Create entity options for the popup
        entity_options = [f"{name} ({color})" for name, pos, color in adjacent_entities]
        
        def popup_callback(selected_option):
            if selected_option:
                # Find the selected entity
                for i, option in enumerate(entity_options):
                    if option == selected_option:
                        name, pos, color = adjacent_entities[i]
                        cls._handle_entity_interaction(game, name, pos, color, root_page)
                        break
        
        # Create and show the popup
        from asciimatics.widgets import PopUpDialog
        popup = PopUpDialog(
            root_page._screen,
            f"Found {len(adjacent_entities)} entities nearby:",
            entity_options,
            popup_callback
        )
        # Add the popup to the current scene
        root_page._screen.current_scene.add_effect(popup)
    
    @classmethod
    def _handle_entity_interaction(cls, game: Game, name: str, pos: VectorN, color: str, root_page: RootPage):
        """Handle interaction with a specific entity."""
        entity = game.world.get_entity(pos)
        
        if hasattr(entity, 'interact'):
            # For interactive entities, show their interaction text
            interaction_text = entity.interact()
            cls._show_interaction_result(name, interaction_text, root_page)
        elif hasattr(entity, 'get_conversation'):
            # For NPCs, start conversation
            cls._start_npc_conversation(game, entity, root_page)
        else:
            # Default interaction
            cls._show_interaction_result(name, f"You interact with {name}.", root_page)
    
    @classmethod
    def _show_interaction_result(cls, name: str, text: str, root_page: RootPage):
        """Show the result of an interaction."""
        def result_callback(selected_option):
            # Just close the result popup
            pass
        
        # Create and show the result popup
        from asciimatics.widgets import PopUpDialog
        popup = PopUpDialog(
            root_page._screen,
            f"Interacting with {name}\n\n{text}",
            ["OK"],
            result_callback
        )
        # Add the popup to the current scene
        root_page._screen.current_scene.add_effect(popup)


def demo(screen: Screen, scene: Scene, game: Game):
    # Create a global variable to store the current dialog
    global current_dialog_scene
    
    scenes = [
        Scene([HelpPage(screen, game)], -1, name="HelpPage"),
        Scene([RootPage(screen, game)], -1, name="RootPage"),
        Scene([MessageLogPage(screen)], -1, name="MessageLogPage"),
        Scene([ExtraPage(screen)], -1, name="ExtraPage"),
    ]
    
    # Add dialog scenes that will be created dynamically
    # These will be added when needed via NextScene

    for scene in scenes[::-1]:
        scene.effects[0].set_theme('bright')

    # Store the current screen dimensions to detect resize
    last_screen_width = screen.width
    last_screen_height = screen.height

    def handle_event(event: Union[KeyboardEvent, MouseEvent]):

        daScene: Scene = screen.current_scene
        daEffects: List[Effect] = daScene.effects

        if len(daEffects) <= 0:
            logging.debug("No effects ;_;")
            return

        maybe_root_page: RootPage = daEffects[0]

        # Check for terminal resize
        nonlocal last_screen_width, last_screen_height
        if (screen.width != last_screen_width or screen.height != last_screen_height):
            logging.debug(f"Terminal resized from {last_screen_width}x{last_screen_height} to {screen.width}x{screen.height}")
            last_screen_width = screen.width
            last_screen_height = screen.height
            
            # Recalculate viewport for new screen size
            if isinstance(maybe_root_page, RootPage):
                maybe_root_page.handle_terminal_resize(screen)

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
        InputHandler.handle_interaction(event, root_page.game, root_page)

        InputHandler.handle_mining(event, root_page.game, root_page)
        root_page.labelInventory.text = root_page.game.player.inventory.summary()

        # after we mine
        root_page.labelFeet.text = str(root_page.game.get_tile_at_player_feet())

        # update viewport display
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

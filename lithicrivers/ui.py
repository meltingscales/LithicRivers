"""
UI components for LithicRivers.
Copyright (c) 2024 Henry Post. All rights reserved.
"""

import logging
from typing import TYPE_CHECKING, Callable, Optional, Union
import platform
import subprocess
import os

import asciimatics.widgets
from asciimatics.event import KeyboardEvent, MouseEvent
from asciimatics.exceptions import NextScene
from asciimatics.scene import Scene
from asciimatics.screen import Canvas, Screen
from asciimatics.widgets import (
    Button,
    Divider,
    Frame,
    Label,
    Layout,
    RadioButtons,
    TextBox,
    _split_text,
)

from lithicrivers.game import NPC, Game, Tile, Tiles
from lithicrivers.keymap import KEYMAP
from lithicrivers.model.model import RenderedData, StopGameError, Viewport
from lithicrivers.model.vector import VectorN
from lithicrivers.settings import DEVELOPER_MODE, GAME_NAME, VIEWPORT_WIGGLE
from lithicrivers.textutil import get_color_for_ui_element, list_label, presenting

if TYPE_CHECKING:
    from asciimatics.effects import Effect

# Global popup manager instance
_popup_manager = None

def get_popup_manager():
    """Get the global popup manager instance."""
    global _popup_manager
    return _popup_manager

def set_popup_manager(manager):
    """Set the global popup manager instance."""
    global _popup_manager
    _popup_manager = manager

def _show_numlock_warning(world_map):
    """Show a warning popup about numlock being off."""
    warning_text = """  NUMLOCK WARNING 

Your NumLock key appears to be turned OFF. This can cause issues with movement controls.

This message will only show once per game session.

When NumLock is OFF:
• Numpad 8 becomes Up Arrow
• Numpad 2 becomes Down Arrow  
• Numpad 4 becomes Left Arrow
• Numpad 6 becomes Right Arrow
• And so on...

To fix this:
1. Press your NumLock key to turn it ON and click OK to continue.
2. The numpad keys should then work normally for movement

You can still use Q/E for up/down movement regardless of NumLock state."""

    def warning_callback(_selected_option):
        # Just close the warning popup
        popup_manager = get_popup_manager()
        if popup_manager:
            popup_manager.set_active_popup(None)

    # Create and show the warning popup
    from asciimatics.widgets import PopUpDialog

    popup = PopUpDialog(
        world_map._screen,
        warning_text,
        ["OK"],
        warning_callback,
    )
    # Track the active popup using popup manager
    popup_manager = get_popup_manager()
    if popup_manager:
        popup_manager.set_active_popup(popup)
    # Add the popup to the current scene
    world_map._screen.current_scene.add_effect(popup)


def get_numlock_state() -> bool:
    """Get the current numlock state using platform-specific APIs."""
    try:
        if platform.system() == "Windows":
            return _get_numlock_state_windows()
        elif platform.system() == "Linux":
            return _get_numlock_state_linux()
        else:
            # For other platforms, assume numlock is on
            return True
    except Exception as e:
        logging.debug(f"Could not detect numlock state: {e}")
        # Assume numlock is on if detection fails
        return True


def _get_numlock_state_windows() -> bool:
    """Get numlock state on Windows using the Windows API."""
    try:
        import ctypes
        hllDll = ctypes.WinDLL("User32.dll")
        VK_NUMLOCK = 0x90
        return bool(hllDll.GetKeyState(VK_NUMLOCK) & 0x0001)
    except Exception as e:
        logging.debug(f"Windows numlock detection failed: {e}")
        return True


def _get_numlock_state_linux() -> bool:
    """Get numlock state on Linux using portable methods."""
    try:
        # Try to use xset first (most portable)
        result = subprocess.run(
            ["xset", "q"], 
            capture_output=True, 
            text=True, 
            timeout=1
        )
        if result.returncode == 0:
            # Look for "Num Lock: on" or "Num Lock: off" in the output
            # xset output can have variable spacing, so use more robust matching
            output = result.stdout.lower()
            import re
            if re.search(r"num lock:\s*on", output):
                return True
            elif re.search(r"num lock:\s*off", output):
                return False
        
        # If xset fails, try to check if we're in a headless environment
        # In headless environments, assume numlock is on (safe default)
        if not os.environ.get('DISPLAY'):
            return True
            
        # Try to use setleds as another fallback
        try:
            result = subprocess.run(
                ["setleds", "-L"], 
                capture_output=True, 
                text=True, 
                timeout=1
            )
            if result.returncode == 0:
                output = result.stdout.lower()
                import re
                if re.search(r"num lock:\s*on", output):
                    return True
                elif re.search(r"num lock:\s*off", output):
                    return False
        except (subprocess.TimeoutExpired, FileNotFoundError):
            pass
            
        # If all methods fail, assume numlock is on (safe default)
        return True
    except Exception as e:
        logging.debug(f"Linux numlock detection failed: {e}")
        return True


def _generate_entity_selection_message(adjacent_entities: list[tuple[str, VectorN, str]]) -> str:
    """Generate a more specific message for entity selection based on entity types."""
    if not adjacent_entities:
        return "No entities nearby."
    
    # Count entity types
    entity_counts = {}
    for name, _, _ in adjacent_entities:
        entity_counts[name] = entity_counts.get(name, 0) + 1
    
    # Generate specific message based on entity types
    if len(entity_counts) == 1:
        entity_name = list(entity_counts.keys())[0]
        count = entity_counts[entity_name]
        if count == 1:
            return f"Found a {entity_name} nearby:"
        else:
            return f"Found {count} {entity_name}s nearby:"
    else:
        # Multiple different entity types
        entity_list = []
        for name, count in entity_counts.items():
            if count == 1:
                entity_list.append(f"a {name}")
            else:
                entity_list.append(f"{count} {name}s")
        
        if len(entity_list) == 2:
            return f"Found {entity_list[0]} and {entity_list[1]} nearby:"
        else:
            # Join with commas and "and" for the last item
            all_but_last = ", ".join(entity_list[:-1])
            return f"Found {all_but_last}, and {entity_list[-1]} nearby:"


def _detect_numlock_issue(event: KeyboardEvent) -> bool:
    """Detect if numlock is off by checking if numpad keys are sending unexpected codes."""
    # When numlock is off, numpad keys send different key codes:
    # Numpad 8 (normally 56) becomes UP_ARROW (-204)
    # Numpad 2 (normally 50) becomes DOWN_ARROW (-205) 
    # Numpad 4 (normally 52) becomes LEFT_ARROW (-206)
    # Numpad 6 (normally 54) becomes RIGHT_ARROW (-207)
    # Numpad 7 (normally 55) becomes HOME (-208)
    # Numpad 9 (normally 57) becomes PAGE_UP (-209)
    # Numpad 1 (normally 49) becomes END (-210)
    # Numpad 3 (normally 51) becomes PAGE_DOWN (-211)
    
    numlock_off_codes = [-204, -205, -206, -207, -208, -209, -210, -211]  # Arrow keys, home, end, page up/down
    return event.key_code in numlock_off_codes


class TabButtons(Layout):
    def __init__(self, frame, game: Game = None):
        # Create buttons list based on developer mode
        buttons = [
            Button("World Map", self._safe_scene_change("WorldMap")),
            Button("Help", self._safe_scene_change("HelpPage")),
            Button("Message Log", self._safe_scene_change("MessageLogPage")),
        ]

        # Add Test Popups button only if developer mode is enabled
        if DEVELOPER_MODE:
            buttons.append(Button("Test Popups", self._safe_scene_change("DevPopupPage")))
            buttons.append(Button("Test Keystrokes", self._safe_scene_change("DevKeystrokesPage")))

        buttons.append(Button("Quit", raise_fn(StopGameError, "Goodbye!")))

        # Create columns based on number of buttons
        cols = [1] * len(buttons)

        super().__init__(cols)

        self._frame = frame
        self.game = game

        for i, _ in enumerate(cols):
            self.add_widget(Divider(), i)

        for i, button in enumerate(buttons):
            self.add_widget(button, i)

    def _safe_scene_change(self, scene_name):
        """Safely change scenes, preventing change if popup is active."""

        def safe_change():
            # Check if there's an active popup
            try:
                popup_manager = get_popup_manager()
                if popup_manager and popup_manager.is_popup_active():
                    # Clear the popup before changing scenes
                    try:
                        if (
                            hasattr(popup_manager.get_active_popup(), "_screen")
                            and popup_manager.get_active_popup()._screen.current_scene
                        ):
                            popup_manager.get_active_popup()._screen.current_scene.remove_effect(
                                popup_manager.get_active_popup()
                            )
                    except Exception:
                        pass
                    popup_manager.set_active_popup(None)
            except NameError:
                # active_popup not defined, safe to proceed
                pass
            # Proceed with scene change
            raise NextScene(scene_name)

        return safe_change


class HeaderLabel(asciimatics.widgets.Widget):
    """
    A text label. But with a header.
    This class was originally made to test how to extend Widget class.
    """

    __slots__ = ["_align", "_required_height", "_text", "header"]

    def __init__(self, label="", height=1, align="<", name=None, header="???"):
        """
        :param label: The text to be displayed for the Label.
        :param height: Optional height for the label.  Defaults to 1 line.
        :param align: Optional alignment for the Label.  Defaults to left aligned.
            Options are "<" = left, ">" = right and "^" = centre
        :param name: The name of this widget.

        """
        # Labels have no value and so should have no name for look-ups either.
        super().__init__(name, tab_stop=False)

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

    def update(self, _frame_no):
        self._frame.canvas: Canvas

        header_prefix = list_label(self.header)

        # Get colors for header and content
        header_color = get_color_for_ui_element("TITLE")

        # Determine content color based on message type
        if self._text.startswith("[ERROR]"):
            content_color = get_color_for_ui_element("ERROR")
        elif self._text.startswith("[SUCCESS]"):
            content_color = get_color_for_ui_element("SUCCESS")
        elif self._text.startswith("[WARNING]"):
            content_color = get_color_for_ui_element("WARNING")
        elif self._text.startswith("[INFO]"):
            content_color = get_color_for_ui_element("INFO")
        elif self._text.startswith("[RARE]"):
            content_color = get_color_for_ui_element("RARE")
        elif self._text.startswith("[VALUABLE]"):
            content_color = get_color_for_ui_element("VALUABLE")
        else:
            content_color = get_color_for_ui_element("LABEL")

        # Render header with title color
        self._frame.canvas.paint(
            header_prefix,
            self._x,
            self._y,
            header_color[0],
            header_color[1],
            header_color[2],
        )

        # Render content with appropriate color
        self._frame.canvas.paint(
            self._text,
            self._x + len(header_prefix),
            self._y,
            content_color[0],
            content_color[1],
            content_color[2],
        )

    def reset(self):
        pass

    def required_height(self, _offset, _width):
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
    __slots__ = ["_align", "_game"]

    def __init__(self, game: Game, align="<", name: Optional[str] = None):
        super().__init__(name, tab_stop=False)

        self.game = game
        self._align = align

        self._frame: Frame

    def required_height(self, _offset, _width):
        # Account for scale: each tile takes up scale characters vertically
        return (
            self.game.viewport.get_height() * self.game.viewport.scale + 2
        )  # +2 for our random text shit

    def required_width(self, _offset, _width):
        # Account for scale: each tile takes up scale characters horizontally
        return self.game.viewport.get_width() * self.game.viewport.scale

    # noinspection PyTypeHints
    def update(self, _frame_no: int):
        self._frame.canvas: Canvas

        content = ""

        # Check if viewport should be visible
        viewport_visible = getattr(self.game, "viewport_visible", True)

        if viewport_visible:
            to_render: RenderedData = self.game.render_world_viewport()
            # Render the world with colors
            self._render_colored_world(to_render)
        # If viewport is hidden, render nothing at all

        # Render the header
        header_color = get_color_for_ui_element("HEADER")
        self._frame.canvas.paint(
            f"{content:{self._align}{self._w}}",
            self._x,
            self._y,
            header_color[0],
            header_color[1],
            header_color[2],
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
                    render_item_chunk = render_item.split("\n")
                    slice = (
                        render_item_chunk[stripe_idx]
                        if stripe_idx < len(render_item_chunk)
                        else " "
                    )
                    slice = slice.replace("\n", "")
                    row_content += slice

                    # Get color for this position and repeat it for each character in the scaled sprite
                    tile_color = rendered_data.get_color_at(x, y)
                    # Repeat the color for each character in the scaled sprite slice
                    for _ in range(len(slice)):
                        row_colors.append(tile_color)

                # Render this row with colors
                self._render_colored_row(
                    row_content,
                    row_colors,
                    start_y + y * rendered_data.scale + stripe_idx,
                )

    def _render_colored_row(
        self, content: str, colors: list[tuple[int, int, int]], y_pos: int
    ):
        """Render a row with individual character colors."""
        x_pos = self._x

        # Clamp content to available width to prevent overflow
        max_width = self._w if hasattr(self, "_w") else len(content)
        clamped_content = content[:max_width]
        clamped_colors = colors[:max_width]

        for i, char in enumerate(clamped_content):
            if i < len(clamped_colors):
                color = clamped_colors[i]
            else:
                color = get_color_for_ui_element("DEFAULT")

            # Paint each character with its color
            self._frame.canvas.paint(
                char, x_pos + i, y_pos, color[0], color[1], color[2]
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


class WorldMap(Frame):
    __slots__ = ["game"]

    def __init__(self, screen, game: Game):
        super().__init__(
            screen, screen.height, screen.width, can_scroll=True, title="World Map"
        )

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

        # Add status bar at the top
        status_layout = Layout([1], fill_frame=False)
        self.add_layout(status_layout)
        self.statusLabel = Label("", name="statusLabel")
        status_layout.add_widget(self.statusLabel)

        # Initialize the status label with current player information
        self.update_status_label()

        layout1 = Layout(columns=columns, fill_frame=True)

        self.add_layout(layout1)

        self.labelPosition = HeaderLabel(name="labelPosition", header="POS")
        layout1.add_widget(self.labelPosition, column=1)
        self.labelPosition.text = str(self.game.render_pretty_player_position())

        self.labelViewport = HeaderLabel(name="labelViewport", header="VIEW")
        layout1.add_widget(self.labelViewport, column=1)
        self.labelViewport.text = str(self.game.viewport.render_pretty())

        self.labelFeet = HeaderLabel(name="labelFeet", header="FEET")
        layout1.add_widget(self.labelFeet, column=1)
        self.labelFeet.text = str(self.game.get_tile_at_player_feet())

        self.labelInventory = HeaderLabel(name="labelInventory", header="INV")
        layout1.add_widget(self.labelInventory, column=1)
        self.labelInventory.text = self.game.player.inventory.colored_summary()

        self.widgetGame = GameWidget(name="widgetGame", game=self.game)
        layout1.add_widget(self.widgetGame, column=0)

        layout_buttons = TabButtons(self)
        self.add_layout(layout_buttons)
        self.fix()

    def update(self, frame_no):
        """Update the status label with current player information."""
        # Call parent update first
        super().update(frame_no)

    def _create_bar(
        self, current: int, maximum: int, label: str, filled: str, empty: str
    ) -> str:
        """Create a visual bar for health/stamina."""
        if maximum <= 0:
            return f"{label}: {current}/{maximum}"

        # Create a 10-character bar
        bar_length = 10
        filled_length = int((current / maximum) * bar_length)
        empty_length = bar_length - filled_length

        bar = filled * filled_length + empty * empty_length
        return f"{bar} {current}/{maximum}"

    def update_status_label(self):
        """Update the status label with current player information."""
        if hasattr(self, "statusLabel") and self.game:
            player = self.game.player
            position = player.position

            # Create status bar content
            health_bar = self._create_bar(player.health, 100, "HP", "█", "░")
            stamina_bar = self._create_bar(player.stamina, 100, "ST", "█", "░")

            # Format the status bar
            status_parts = [
                f"Health: {health_bar}",
                f"Stamina: {stamina_bar}",
                f"Position: {position.as_short_string()}",
                f"Tile: {self.game.get_tile_at_player_feet().tileid}",
                f"Scale: {self.game.viewport.scale}x",
            ]

            # Join with separators
            self.statusLabel.text = " | ".join(status_parts)

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

        # Account for borders, headers, tab buttons, and status bar
        available_width = screen_width - info_panel_width - 4  # 4 for borders
        available_height = (
            screen_height - 7
        )  # 7 for headers, borders, tab buttons, and status bar (1 line)

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
            scale=self.game.viewport.scale,
        )
        self.game.viewport = new_viewport

    def handle_terminal_resize(self, screen):
        """Handle terminal resize by recalculating viewport size."""
        self._adjust_viewport_for_screen(screen)
        # Update viewport display
        self.labelViewport.text = str(self.game.viewport.render_pretty())


class HelpPage(Frame):
    def __init__(self, screen, game: Game):
        super().__init__(
            screen, screen.height, screen.width, can_scroll=False, title="Help"
        )
        self.game = game
        layout1 = Layout([1], fill_frame=True)
        self.add_layout(layout1)
        # add your widgets here

        helptxt = (
            f"Hello! Welcome to {GAME_NAME}. Below are keys.\n"
            "By the way, game UI nav is arrow keys + space or enter.\n"
            "You can also use the mouse! Left click works!\n"
            "Press ESC to return to the game.\n"
            "Enjoy!\n"
            "\n"
            f"Your character's appearance: {presenting(game.player.render_sprite(1))}\n"
            "\n"
            "=== KEYBINDS ===\n"
        )

        helptxt += KEYMAP.generate_categorized_key_guide()

        helptxtheight = len(helptxt.split("\n"))

        help_label = Label(helptxt, height=helptxtheight, name="helpLabel")

        layout1.add_widget(help_label)

        layout2 = TabButtons(self)
        self.add_layout(layout2)
        self.fix()

    def process_event(self, event):
        """Handle events for the help page, including ESC to close."""
        # Check for ESC key to close help menu using keymap
        if hasattr(event, 'key_code') and KEYMAP.matches("CLOSE_HELP_MENU", event):
            raise NextScene("WorldMap")
        
        # Let the parent class handle other events
        return super().process_event(event)


class MessageLogPage(Frame):
    def __init__(self, screen, game: Game = None):
        super().__init__(
            screen, screen.height, screen.width, can_scroll=False, title="Message Log"
        )
        self.game = game

        # Create main layout for messages - use full width
        layout1 = Layout([1], fill_frame=True)
        self.add_layout(layout1)

        # Create message display widget using Label for better width handling
        from asciimatics.widgets import Label

        self.message_display = Label(
            "",  # Initial empty text
            height=screen.height - 4,  # Leave room for tab buttons
            name="message_display",
        )
        layout1.add_widget(self.message_display)

        # Create tab buttons
        layout2 = TabButtons(self)
        self.add_layout(layout2)

        self.fix()
        
        # Initialize the message display immediately
        self.update_messages()

    def reset(self):
        """Reset the frame and ensure message display is properly initialized."""
        super().reset()
        # Force update of messages when frame is reset/shown
        self.update_messages()

    def update_messages(self):
        """Update the message display with current messages."""
        if not self.game or not self.game.message_log:
            return

        messages = self.game.message_log.get_recent_messages(
            50
        )  # Show last 50 messages
        if not messages:
            self.message_display.text = (
                "No messages yet.\n\nStart playing to see your actions logged here!"
            )
            return

        # Format messages for display
        formatted_messages = []
        for msg in messages:
            timestamp = msg["timestamp"]
            gametick = msg.get("gametick", 0)  # Get gametick, default to 0 for backward compatibility
            message_type = msg["type"]
            message = msg["message"]

            # Add color coding based on message type
            type_icon = {
                "mining": "⛏️ ",
                "interaction": "💬 ",
                "pickup": "📦 ",
                "dialog": "🗣️ ",
                "info": "ℹ️ ",
            }.get(message_type, "• ")

            formatted_messages.append(f"[{timestamp} T{gametick}] {type_icon}{message}")

        # Join all messages with newlines
        self.message_display.text = "\n".join(formatted_messages)

    def update(self, frame_no):
        """Update the frame, refreshing messages."""
        super().update(frame_no)
        self.update_messages()


class DevKeystrokesPage(Frame):
    def __init__(self, screen):
        super().__init__(
            screen, screen.height, screen.width, can_scroll=False, title="Test Keystrokes"
        )

        layout1 = Layout([1], fill_frame=True)
        self.add_layout(layout1)

        # Add a big text box that contains a running log of the last 10 keystrokes and their int codes as well as ascii-printable representations (if they can be printed)

        self.textBoxKeystrokes = Label("keystrokes", height=20)
        self.logKeystrokes = list()

        layout1.add_widget(self.textBoxKeystrokes)

        self.render_log()

        buttons = TabButtons(self)
        self.add_layout(buttons)

        self.fix()

    def render_log(self):
        self.textBoxKeystrokes.text = '\n'.join(self.logKeystrokes)

    def append_to_log(self,m:str):
        if len(self.logKeystrokes) > 10:
            del self.logKeystrokes[0]

        self.logKeystrokes.append(m)

    def update_keystroke(self, event: Union[KeyboardEvent,MouseEvent]):

        if isinstance(event, MouseEvent):

            self.append_to_log(f"mouse event TODO process it: {repr(event)}")
            pass #TODO: For now, we're ignoring MouseEvent.

        if isinstance(event, KeyboardEvent):
            event:KeyboardEvent

            key_code = event.key_code

            # try to get ascii representation and remove whitespace
            try:
                ascii_rep = chr(key_code)
                if ascii_rep.strip() == "":
                    ascii_rep = f"0x{key_code:02x}"
            except ValueError:
                ascii_rep = f"0x{key_code:02x}"

            self.append_to_log(f"test {key_code} {ascii_rep}")
            self.render_log()
            self.fix()


class DevPopupPage(Frame):
    def __init__(self, screen):
        super().__init__(
            screen, screen.height, screen.width, can_scroll=False, title="Test Popups"
        )
        layout1 = Layout([1], fill_frame=True)
        self.add_layout(layout1)

        # Add test popup buttons
        from asciimatics.widgets import Button, Label

        def test_simple_dialog():
            """Test a simple dialog without options."""

            def callback(result):
                print(f"Simple dialog result: {result}")
                popup_manager = get_popup_manager()
                if popup_manager:
                    popup_manager.set_active_popup(None)

            # Use asciimatics PopUpDialog for simple dialog
            from asciimatics.widgets import PopUpDialog

            popup = PopUpDialog(
                screen,
                "This is a test dialog with no options.\nPress OK to continue.",
                ["OK"],
                callback,
            )
            # Track the active popup using popup manager
            popup_manager = get_popup_manager()
            if popup_manager:
                popup_manager.set_active_popup(popup)
            # Add the popup to the current scene
            screen.current_scene.add_effect(popup)

        def test_options_dialog():
            """Test a dialog with options."""

            def callback(result):
                print(f"Options dialog result: {result}")
                popup_manager = get_popup_manager()
                if popup_manager:
                    popup_manager.set_active_popup(None)

            # Use asciimatics PopUpDialog for options dialog
            from asciimatics.widgets import PopUpDialog

            popup = PopUpDialog(
                screen,
                "This is a test dialog with options.\nSelect an option:",
                ["Option 1", "Option 2", "Option 3", "Option 4"],
                callback,
            )
            # Track the active popup using popup manager
            popup_manager = get_popup_manager()
            if popup_manager:
                popup_manager.set_active_popup(popup)
            # Add the popup to the current scene
            screen.current_scene.add_effect(popup)

        def test_large_dialog():
            """Test a large dialog with lots of content."""

            def callback(result):
                print(f"Large dialog result: {result}")
                popup_manager = get_popup_manager()
                if popup_manager:
                    popup_manager.set_active_popup(None)

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
                callback,
            )
            # Track the active popup using popup manager
            popup_manager = get_popup_manager()
            if popup_manager:
                popup_manager.set_active_popup(popup)
            # Add the popup to the current scene
            screen.current_scene.add_effect(popup)

        def test_popup_box():
            """Test asciimatics PopUpDialog."""
            from asciimatics.widgets import PopUpDialog

            def callback(result):
                print(f"Popup dialog result: {result}")
                popup_manager = get_popup_manager()
                if popup_manager:
                    popup_manager.set_active_popup(None)

            # Create a popup dialog using asciimatics PopUpDialog
            popup = PopUpDialog(
                screen,
                "This is a test popup dialog.\n\n"
                "This uses the built-in asciimatics PopupDialog widget.\n"
                "It should work much better than our custom implementation.",
                ["OK", "Cancel"],
                callback,
            )
            # Track the active popup using popup manager
            popup_manager = get_popup_manager()
            if popup_manager:
                popup_manager.set_active_popup(popup)
            # Add the popup to the current scene
            screen.current_scene.add_effect(popup)

        # Add buttons to test different dialog types
        layout1.add_widget(Button("Test Simple Dialog", test_simple_dialog))
        layout1.add_widget(Button("Test Options Dialog", test_options_dialog))
        layout1.add_widget(Button("Test Large Dialog", test_large_dialog))
        layout1.add_widget(Button("Test Popup Dialog", test_popup_box))

        # Add info text
        info_label = Label(
            "Click the buttons above to test different dialog types.\n"
            "These use proper asciimatics components instead of custom widgets.\n"
            "Press 'v' in the main game to toggle viewport visibility."
        )
        layout1.add_widget(info_label)

        layout2 = TabButtons(self)
        self.add_layout(layout2)
        self.fix()


class DialogBox(Frame):
    """A modal dialog box for conversations and interactions."""

    def __init__(
        self,
        screen,
        title: str,
        content: str,
        options: Optional[list[str]] = None,
        callback: Optional[Callable] = None,
        game: Game = None,
    ):
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
            screen, height, max_width, title=title, can_scroll=False, has_border=True
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
                options, label="Choose an option:", name="options"
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
        if hasattr(self, "_screen") and self._screen.current_scene:
            self._screen.current_scene.remove_effect(self)


class EntitySelectionPopup(Frame):
    """A modal popup for selecting which entity to interact with."""

    def __init__(
        self, screen, game: Game, adjacent_entities: list[tuple[str, VectorN, str]]
    ):
        # Calculate popup size and position
        max_width = min(60, screen.width - 4)
        height = len(adjacent_entities) + 8  # +8 for header, buttons, borders, etc.

        # Center the popup on screen
        (screen.width - max_width) // 2
        (screen.height - height) // 2

        super().__init__(
            screen,
            height,
            max_width,
            title="Choose Entity to Interact With",
            can_scroll=False,
            has_border=True,
        )

        self.game = game
        self.adjacent_entities = adjacent_entities
        self.screen = screen

        # Create layout
        layout = Layout([1], fill_frame=True)
        self.add_layout(layout)

        # Add header
        header = Label(
            _generate_entity_selection_message(adjacent_entities), name="header"
        )
        layout.add_widget(header)

        # Add entity options
        self.entity_widget = RadioButtons(
            [f"{name} ({color})" for name, pos, color in adjacent_entities],
            label="Select an entity:",
            name="entities",
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

    def _handle_interaction(self, name: str, pos: VectorN, _color: str):
        """Handle the actual interaction."""
        # Get the entity
        entity = self.game.world.get_entity(pos)

        if hasattr(entity, "interact"):
            # For interactive entities, show their interaction text
            interaction_text = entity.interact()
            self._show_interaction_result(name, interaction_text)
        elif hasattr(entity, "get_conversation"):
            # For NPCs, start conversation
            self._start_npc_conversation(entity)
        else:
            # Default interaction
            self._show_interaction_result(name, f"You interact with {name}.")

    def _show_interaction_result(self, name: str, text: str):
        """Show the result of an interaction."""
        # Create a result popup
        InteractionResultPopup(self.screen, f"Interacting with {name}", text)
        # For now, just show the result in the message area
        # TODO: Implement proper result popup display

    def _start_npc_conversation(self, npc):
        """Start a conversation with an NPC."""
        npc.get_conversation()
        # For now, just show the conversation in the message area
        # TODO: Implement proper conversation dialog
        pass

    def _close(self):
        """Close the popup and return to the game."""
        # Remove this popup from the current scene
        if hasattr(self, "_screen") and self._screen.current_scene:
            self._screen.current_scene.remove_effect(self)


class InteractionResultPopup(Frame):
    """A popup to show the result of an interaction."""

    def __init__(self, screen, title: str, content: str):
        # Calculate popup size
        max_width = min(70, screen.width - 4)
        lines = _split_text(content, max_width - 4, 10)
        height = len(lines) + 6  # +6 for title, buttons, borders

        super().__init__(
            screen, height, max_width, title=title, can_scroll=False, has_border=True
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
        if hasattr(self, "_screen") and self._screen.current_scene:
            self._screen.current_scene.remove_effect(self)


class InputHandler:
    @staticmethod
    def handle_movement(keyboard_event: KeyboardEvent) -> Union[None, VectorN]:
        """
        :param keyboardEvent:
        :return: Vector the input resolves to.
        """

        # check for movement key
        if KEYMAP.matches_movement_key(keyboard_event):
            return KEYMAP.get_movement_vector(keyboard_event)

        return None

    @staticmethod
    def handle_mining(event: KeyboardEvent, game: Game, world_map: WorldMap):
        # TODO: clean up state... :P why do we pass all these as args?

        if not KEYMAP.matches("MINE", event):
            return

        tile_under: Tile = game.get_tile_at_player_feet()
        if tile_under == Tiles.dirt():
            return  # can't mine dirt
        elif tile_under == Tiles.tree():
            # Allow mining trees - they drop guaranteed acorns plus other items
            tree_drops = tile_under.calc_tree_drops()
            dropped_items = []
            for item in tree_drops:
                game.player.inventory.add_item(item)
                dropped_items.append(item.name)
            game.set_tile_at_player_feet(Tiles.dirt())
            game.log_mining("tree", dropped_items)
            game.increment_tick()
            world_map.update_status_label()
        elif tile_under == Tiles.gold_ore():
            dropped_item = tile_under.calc_drop()
            game.player.inventory.add_item(dropped_item)
            game.set_tile_at_player_feet(Tiles.dirt())
            game.log_mining("gold ore", [dropped_item.name])
            game.increment_tick()
            world_map.update_status_label()

    @classmethod
    def handle_viewport(
        cls, event: KeyboardEvent, game: Game, world_map: WorldMap = None
    ):
        if KEYMAP.matches("RESET_VIEWPORT", event):
            game.reset_viewport()
            if world_map:
                world_map.update_status_label()

        if KEYMAP.matches("SLIDE_VIEWPORT_WEST", event):
            game.viewport.slide_left()
            if world_map:
                world_map.update_status_label()

        if KEYMAP.matches("SLIDE_VIEWPORT_EAST", event):
            game.viewport.slide_right()
            if world_map:
                world_map.update_status_label()

        if KEYMAP.matches("TOGGLE_VIEWPORT", event):
            # Toggle viewport visibility by setting a flag
            if not hasattr(game, "viewport_visible"):
                game.viewport_visible = True
            game.viewport_visible = not game.viewport_visible
            if world_map:
                world_map.update_status_label()

    @classmethod
    def handle_scale(cls, event, game, world_map: WorldMap = None):
        if KEYMAP.matches("SCALE_DOWN", event):
            game.viewport.rescale_down(1)
            game.reset_viewport()
            if world_map:
                world_map.update_status_label()

        if KEYMAP.matches("SCALE_UP", event):
            game.viewport.rescale_up(1)
            game.reset_viewport()
            if world_map:
                world_map.update_status_label()

    @classmethod
    def handle_interaction(cls, event: KeyboardEvent, game: Game, world_map: WorldMap):
        """Handle interaction with adjacent entities."""
        if not KEYMAP.matches("INTERACT", event):
            return

        # Get adjacent entities
        adjacent_entities = game.world.get_adjacent_entities(game.player.position)

        if not adjacent_entities:
            return

        # Show interaction popup for entities
        cls._show_interaction_popup(game, adjacent_entities, world_map)

    @classmethod
    def _start_npc_conversation(cls, game: Game, npc: NPC, world_map: WorldMap):
        """Start a conversation with an NPC."""
        cls._show_npc_conversation(game, npc, "greeting", world_map)

    @classmethod
    def _show_npc_conversation(
        cls, game: Game, npc: NPC, topic: str, world_map: WorldMap
    ):
        """Show an NPC conversation for a specific topic."""
        conversation = npc.get_conversation(topic)
        logging.debug(
            f"NPC conversation: showing topic '{topic}' with options: {conversation['options']}"
        )

        # Log the NPC's conversation text
        game.log_dialog(npc.name, conversation["text"])

        def conversation_callback(selected_option):
            popup_manager = get_popup_manager()
            logging.debug(f"NPC conversation callback called with: '{selected_option}'")
            if selected_option is not None:
                # Handle the selected option - PopUpDialog returns the index, so we need to get the actual text
                if isinstance(selected_option, int) and 0 <= selected_option < len(
                    conversation["options"]
                ):
                    response = conversation["options"][selected_option]
                else:
                    response = selected_option
                logging.debug(
                    f"NPC conversation: selected '{response}' from topic '{topic}'"
                )

                # Log the player's response
                game.log_dialog("You", response)

                next_topic = npc.handle_response(response, topic)
                logging.debug(
                    f"NPC conversation: next_topic='{next_topic}', current_topic='{topic}'"
                )
                if next_topic and next_topic != topic:
                    # Continue the conversation with the next topic
                    logging.debug(
                        f"NPC conversation: continuing to topic '{next_topic}'"
                    )
                    # Clear the current popup first
                    popup_manager.set_active_popup(None)
                    cls._show_npc_conversation(game, npc, next_topic, world_map)
                else:
                    # No next topic or same topic, close the conversation
                    logging.debug(
                        f"NPC conversation: closing - next_topic='{next_topic}', current_topic='{topic}'"
                    )
                    popup_manager.set_active_popup(None)
            else:
                # No option selected, close the conversation
                logging.debug("NPC conversation: no option selected, closing")
                popup_manager.set_active_popup(None)

        # Show the conversation in a popup
        from asciimatics.widgets import PopUpDialog

        logging.debug(
            f"NPC conversation: creating popup with text: '{conversation['text'][:50]}...'"
        )
        popup = PopUpDialog(
            world_map._screen,
            conversation["text"],
            conversation["options"],
            conversation_callback,
        )
        # Track the active popup using popup manager
        popup_manager = get_popup_manager()
        if popup_manager:
            popup_manager.set_active_popup(popup)
        logging.debug("NPC conversation: popup created, adding to scene")
        # Add the popup to the current scene
        world_map._screen.current_scene.add_effect(popup)

    @classmethod
    def _show_interaction_popup(
        cls,
        game: Game,
        adjacent_entities: list[tuple[str, VectorN, str]],
        world_map: WorldMap,
    ):
        """Show interaction popup for entities."""
        # Create entity options for the popup
        entity_options = [f"{name} ({color})" for name, pos, color in adjacent_entities]

        def popup_callback(selected_option: int):
            """Handle the selected option."""
            if selected_option is not None:

                # Find the selected entity
                chosen_entity = adjacent_entities[selected_option]
                name, pos, color = chosen_entity
                cls._handle_entity_interaction(
                    game, name, pos, color, world_map
                )

            popup_manager = get_popup_manager()
            if popup_manager:
                popup_manager.set_active_popup(None)

        # Create and show the popup
        from asciimatics.widgets import PopUpDialog

        popup = PopUpDialog(
            world_map._screen,
            _generate_entity_selection_message(adjacent_entities),
            entity_options,
            popup_callback,
        )
        # Track the active popup using popup manager
        popup_manager = get_popup_manager()
        if popup_manager:
            popup_manager.set_active_popup(popup)
        # Add the popup to the current scene
        world_map._screen.current_scene.add_effect(popup)

    @classmethod
    def _handle_entity_interaction(
        cls, game: Game, name: str, pos: VectorN, _color: str, world_map: WorldMap
    ):
        """Handle interaction with a specific entity."""
        entity = game.world.get_entity(pos)

        if hasattr(entity, "interact"):
            # For interactive entities, show their interaction text
            interaction_text = entity.interact()
            game.log_interaction(name, interaction_text)
            game.increment_tick()
            cls._show_interaction_result(name, interaction_text, world_map)
        elif hasattr(entity, "get_conversation"):
            # For NPCs, start conversation
            game.log_interaction(name, "Started conversation")
            game.increment_tick()
            cls._start_npc_conversation(game, entity, world_map)
        else:
            # Default interaction
            default_text = f"You interact with {name}."
            game.log_interaction(name, default_text)
            game.increment_tick()
            cls._show_interaction_result(name, default_text, world_map)

    @classmethod
    def _show_interaction_result(cls, name: str, text: str, world_map: WorldMap):
        """Show the result of an interaction."""

        def result_callback(_selected_option):
            # Just close the result popup
            popup_manager = get_popup_manager()
            if popup_manager:
                popup_manager.set_active_popup(None)

        # Create and show the result popup
        from asciimatics.widgets import PopUpDialog

        popup = PopUpDialog(
            world_map._screen,
            f"Interacting with {name}\n\n{text}",
            ["OK"],
            result_callback,
        )
        # Track the active popup using popup manager
        popup_manager = get_popup_manager()
        if popup_manager:
            popup_manager.set_active_popup(popup)
        # Add the popup to the current scene
        world_map._screen.current_scene.add_effect(popup)


class PopupManager:
    """Manages popup dialogs and their lifecycle."""
    
    def __init__(self):
        self.active_popup = None
        self.numlock_warning_shown = False
    
    def set_active_popup(self, popup):
        """Set the currently active popup."""
        self.active_popup = popup
    
    def get_active_popup(self):
        """Get the currently active popup."""
        return self.active_popup
    
    def is_popup_active(self):
        """Check if there's an active popup."""
        return self.active_popup is not None
    
    def close_active_popup(self, screen):
        """Close the currently active popup."""
        if self.active_popup is not None:
            try:
                if (
                    hasattr(self.active_popup, "_screen")
                    and self.active_popup._screen.current_scene
                ):
                    self.active_popup._screen.current_scene.remove_effect(self.active_popup)
            except Exception as e:
                logging.info(f"Error closing popup: {e}")
            finally:
                self.active_popup = None
    
    def handle_esc_key(self, screen):
        """Handle ESC key press to close active popup."""
        if self.active_popup is not None:
            self.close_active_popup(screen)
            return True
        return False
    
    def handle_popup_event(self, event, screen):
        """Handle events for the active popup."""
        if self.active_popup is not None:
            # Check if popup is still in the current scene
            if self.active_popup not in screen.current_scene.effects:
                # Popup was removed from scene, clear it
                self.active_popup = None
                return None  # No popup to handle
            
            # Let the popup handle the event
            try:
                result = self.active_popup.process_event(event)
                if result is None:  # Event was handled by popup
                    return False  # Event was handled, don't continue processing
                return None  # Event was not handled by popup, continue processing
            except Exception as e:
                # Popup had an error, clear it
                logging.info(f"Popup error: {e}")
                self.active_popup = None
                return None  # No popup to handle
        return None  # No active popup
    
    def clear_popup_on_page_switch(self, screen):
        """Clear any active popup when switching to non-World Map pages."""
        if self.active_popup is not None:
            try:
                if (
                    hasattr(self.active_popup, "_screen")
                    and self.active_popup._screen.current_scene
                ):
                    self.active_popup._screen.current_scene.remove_effect(self.active_popup)
            except Exception:
                pass
            self.active_popup = None
    
    def handle_numlock_warning(self, world_map, screen):
        """Handle numlock warning logic."""
        # Check numlock state during first interaction (proactive detection)
        if not self.numlock_warning_shown and not get_numlock_state():
            # Show warning immediately if numlock is off
            _show_numlock_warning(world_map)
            self.numlock_warning_shown = True
            return True  # Don't process movement until user acknowledges warning
        
        # Check if numlock has been toggled on (state changed from off to on)
        if self.active_popup is not None and get_numlock_state():
            # Close the popup if numlock is now on
            self.close_active_popup(screen)
            return True  # Process the movement after closing popup
        
        return False
    
    def check_numlock_issue(self, event, world_map):
        """Check for numlock issues and show warning if needed."""
        if _detect_numlock_issue(event):
            # Only show warning if no popup is currently active and warning hasn't been shown
            if self.active_popup is None and not self.numlock_warning_shown:
                _show_numlock_warning(world_map)
                self.numlock_warning_shown = True
            return True  # Don't process movement when numlock is off
        return False


def demo(screen: Screen, scene: Scene, game: Game):
    # Create a global variable to store the current dialog
    global current_dialog_scene

    # Create popup manager to handle all popup-related logic
    popup_manager = PopupManager()
    set_popup_manager(popup_manager)  # Set the global instance
    
    # Global variable to track if numlock warning has been shown
    global numlock_warning_shown
    numlock_warning_shown = False  # Initialize to False
    

    
    scenes = [
        Scene([WorldMap(screen, game)], -1, name="WorldMap"),
        Scene([HelpPage(screen, game)], -1, name="HelpPage"),
        Scene([MessageLogPage(screen, game)], -1, name="MessageLogPage"),
        Scene([DevPopupPage(screen)], -1, name="DevPopupPage"),
        Scene([DevKeystrokesPage(screen)], -1, name="DevKeystrokesPage"),
    ]

    # Add dialog scenes that will be created dynamically
    # These will be added when needed via NextScene

    for scene in scenes[::-1]:
        scene.effects[0].set_theme("bright")

    # Store the current screen dimensions to detect resize
    last_screen_width = screen.width
    last_screen_height = screen.height

    def handle_event(event: Union[KeyboardEvent, MouseEvent]):
        current_scene: Scene = screen.current_scene
        current_effects: list[Effect] = current_scene.effects

        if len(current_effects) <= 0:
            logging.debug("No effects ;_;")
            return

        # This is the topmost effect. It may or may not be the world map. We need to find that out first.
        current_effect: WorldMap = current_effects[0]

        # Check for terminal resize
        nonlocal last_screen_width, last_screen_height
        if screen.width != last_screen_width or screen.height != last_screen_height:
            logging.debug(
                f"Terminal resized from {last_screen_width}x{last_screen_height} to {screen.width}x{screen.height}"
            )
            last_screen_width = screen.width
            last_screen_height = screen.height

            # Recalculate viewport for new screen size
            if isinstance(current_effect, WorldMap):
                current_effect.handle_terminal_resize(screen)

        # TODO: Why do we ignore all non-KeyboardEvent objects? This is going to need to be removed if we ever want to handle mouse inputs natively.
        if not isinstance(event, KeyboardEvent):
            # print("not keyboard event, ignoring... - {}".format(event))
            return
        event: KeyboardEvent

        # We want to display the KeyboardEvent on the DevKeystrokesPage
        if isinstance(current_effect, DevKeystrokesPage):
            current_effect: DevKeystrokesPage
            current_effect.update_keystroke(event)
            return

        # Check for ESC key to close popups
        if KEYMAP.matches("CLOSE_HELP_MENU", event):
            if popup_manager.handle_esc_key(screen):
                return

        # Handle popup events
        popup_result = popup_manager.handle_popup_event(event, screen)
        if popup_result is False:  # Event was handled by popup and should not continue
            return

        # TODO: This is a pretty gross way of handling this. We should have a second handler function that just dispatches the event to a specific panel.
        if current_effect.title.strip() != "World Map":
            logging.info("Not supposed to handle " + current_effect.title)
            # Clear any active popup when switching to non-World Maps
            popup_manager.clear_popup_on_page_switch(screen)
            return

        world_map = current_effect

        # Handle numlock warning logic
        if popup_manager.handle_numlock_warning(world_map, screen):
            return  # Don't process movement until user acknowledges warning
        
        # Check for numlock issues before handling movement (fallback detection)
        if popup_manager.check_numlock_issue(event, world_map):
            return  # Don't process movement when numlock is off

        move_vec = InputHandler.handle_movement(event)
        if move_vec:
            world_map.game.move_player(move_vec)

            # display pos
            world_map.labelPosition.text = game.render_pretty_player_position()

            # Update status label immediately
            world_map.update_status_label()

            # move the viewport with the player
            if game.player_outside_viewport(wiggle=VIEWPORT_WIGGLE):
                game.viewport.slide(move_vec)

                # still outside? Something's wrong, let's reset the viewport...
                if game.player_outside_viewport(wiggle=VIEWPORT_WIGGLE):
                    game.reset_viewport()

        InputHandler.handle_viewport(event, world_map.game, world_map)
        InputHandler.handle_scale(event, world_map.game, world_map)
        InputHandler.handle_interaction(event, world_map.game, world_map)

        InputHandler.handle_mining(event, world_map.game, world_map)
        world_map.labelInventory.text = world_map.game.player.inventory.summary()

        # after we mine
        world_map.labelFeet.text = str(world_map.game.get_tile_at_player_feet())

        # update viewport display
        world_map.labelViewport.text = str(world_map.game.viewport.render_pretty())

    screen.set_title(f"~~-[ {GAME_NAME} ]-~~")
    screen.play(
        scenes,
        stop_on_resize=True,
        start_scene=scene,
        allow_int=True,
        unhandled_input=handle_event,
    )


def _raise(ex):
    """Because we can't use raise in lambda for some reason..."""
    raise ex


def raise_fn(clazz: any, name: str):
    """
    bruh, why?
    """

    def raise_next_scene(arg=name):
        raise clazz(arg)

    return raise_next_scene

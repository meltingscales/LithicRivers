"""
TUI popup dialog components for LithicRivers.
Copyright (c) 2024 Henry Post. All rights reserved.
"""

from functools import partial
from inspect import isfunction
from typing import Callable, Optional

from asciimatics.widgets import Button, Frame, Layout, TextBox, _split_text


class VerticalPopUpDialog(Frame):
    """
    A vertical version of PopUpDialog that arranges buttons vertically instead of horizontally.
    
    This class provides a modal dialog with vertically arranged buttons, which is often
    more user-friendly than horizontal button layouts, especially for multiple options
    or longer button text.
    """

    def __init__(
        self, 
        screen, 
        text: str, 
        buttons: list[str], 
        on_close: Optional[Callable] = None, 
        has_shadow: bool = False, 
        theme: str = "warning"
    ):
        """
        Initialize the vertical popup dialog.
        
        Args:
            screen: The Screen that owns this dialog.
            text: The message text to display.
            buttons: A list of button names to display. This may be an empty list.
            on_close: Optional function to invoke on exit.
            has_shadow: Optional flag to specify if dialog should have a shadow when drawn.
            theme: Optional colour theme for this pop-up. Defaults to the warning colours.

        The `on_close` method (if specified) will be called with one integer parameter that
        corresponds to the index of the button passed in the array of available `buttons`.

        Note that `on_close` must be a static method to work across screen resizing. Either it
        is static (and so the dialog will be cloned) or it is not (and the dialog will disappear
        when the screen is resized).
        """
        # Remember parameters for cloning.
        self._text = text
        self._buttons = buttons
        self._on_close = on_close

        # Decide on optimum width of the dialog. Limit to 2/3 the screen width.
        string_len = getattr(screen, 'unicode_aware', False) and (lambda x: len(x)) or len
        width = max(string_len(x) for x in text.split("\n"))
        # For vertical buttons, we need to account for the widest button
        if buttons:
            max_button_width = max(string_len(x) for x in buttons)
            width = max(width + 2, max_button_width + 4)
        width = min(width, screen.width * 2 // 3)

        # Figure out the necessary message and allow for buttons and borders
        # when deciding on height.
        delta_h = 4 + len(buttons) if len(buttons) > 0 else 2  # Extra height for vertical buttons
        self._message = _split_text(text, width - 2, screen.height - delta_h, getattr(screen, 'unicode_aware', False))
        height = len(self._message) + delta_h

        # Construct the Frame
        self._data = {"message": self._message}
        super().__init__(
            screen, height, width, self._data, has_shadow=has_shadow, is_modal=True)

        # Build up the message box
        layout = Layout([width - 2], fill_frame=True)
        self.add_layout(layout)
        text_box = TextBox(len(self._message), name="message")
        text_box.disabled = True
        layout.add_widget(text_box)
        
        # Add vertical button layout
        if buttons:
            layout2 = Layout([1])  # Single column for vertical buttons
            self.add_layout(layout2)
            for i, button in enumerate(buttons):
                func = partial(self._destroy, i)
                layout2.add_widget(Button(button, func))
        
        self.fix()

        # Ensure that we have the right palette in place
        self.set_theme(theme)

    def _destroy(self, selected: int) -> None:
        """Destroy the dialog and call the on_close callback if provided."""
        self._scene.remove_effect(self)
        if self._on_close:
            self._on_close(selected)

    def clone(self, screen, scene) -> None:
        """
        Create a clone of this Dialog into a new Screen.

        Args:
            screen: The new Screen object to clone into.
            scene: The new Scene object to clone into.
        """
        # Only clone the object if the function is safe to do so.
        if self._on_close is None or isfunction(self._on_close):
            scene.add_effect(VerticalPopUpDialog(screen, self._text, self._buttons, self._on_close)) 
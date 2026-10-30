# game-window Specification

## Purpose
The game's own window: it draws the same text UI as the terminal with fonts the game ships, takes keyboard and mouse input, and resizes, so the game looks the same on every machine without a terminal emulator.

## Requirements

### Requirement: Window by default
Running the client without `--terminal`, `--script`, `--bench` or `--mcp` SHALL open a game window and draw the game in it. Running it with `--terminal` SHALL use the current terminal as before. Closing the window SHALL quit like the game's Quit choice.

#### Scenario: Normal launch
- **WHEN** a player starts the game from Steam or a desktop shortcut
- **THEN** a game window opens showing the splash screens, with no terminal emulator involved

#### Scenario: Terminal mode
- **WHEN** a player starts the client with `--terminal` from a terminal
- **THEN** the game runs in that terminal exactly as before this change

#### Scenario: Closing the window
- **WHEN** the player closes the window
- **THEN** the game exits as if Quit were chosen

### Requirement: Bundled fonts
The window SHALL draw text with a monospace font shipped with the game, and SHALL NOT depend on fonts installed on the system. Every character the game can draw (sprites, UI text, box drawing, art, dialogue effects) SHALL be covered by the bundled fonts, and no bundled font SHALL make the screen cells narrower than the main font.

#### Scenario: Coverage
- **WHEN** the font coverage test runs
- **THEN** every character used by sprites, UI strings and art assets is found in a bundled font, or the test names the missing ones

### Requirement: Same input as the terminal
Key presses in the window SHALL reach the game as the same key codes the terminal produces, so every keybind works unchanged. Mouse clicks, scrolls and moves SHALL be reported at the screen cell under the pointer, with Shift when held, the same way terminal mouse events are.

#### Scenario: A keybind
- **WHEN** the player presses the key bound to `movement.MOVE_EAST` in the window
- **THEN** the player moves east, as in the terminal

#### Scenario: Clicking a tile
- **WHEN** the player left-clicks a world tile in the window
- **THEN** the click is handled at that tile's cell, as a terminal click there would be

### Requirement: Resizing and text size
The game window SHALL fit as many columns and rows as its size and the text size allow, and recompute them when the window is resized or the text size changes. Text size SHALL change in steps between a minimum and a maximum. The text size and fullscreen choice SHALL be saved in the config file and restored at the next launch.

#### Scenario: Resizing
- **WHEN** the player makes the game window wider
- **THEN** the game lays out with more columns on the next frame

#### Scenario: Bigger text persists
- **WHEN** the player makes the text bigger twice, quits, and starts the game again
- **THEN** the game window opens with the bigger text

### Requirement: Display panel
The game window SHALL offer a Display panel with buttons to make the text smaller, make it bigger, and turn fullscreen on or off, showing the current text size. Clicking a button SHALL apply the change on the next frame and save it. In terminal mode there SHALL be no Display panel.

#### Scenario: Bigger text from the Display panel
- **WHEN** the player clicks the bigger-text button
- **THEN** the game window's text gets one step bigger, the panel shows the new size, and the screen shows no leftover text from the old size

#### Scenario: Fullscreen from the Display panel
- **WHEN** the player clicks the fullscreen button while the game window is windowed
- **THEN** the game window goes fullscreen, and clicking it again returns it to a window

### Requirement: Display panel placement
The Display panel SHALL be docked, floating or hidden, chosen in Options and saved. Docked, the default, is a strip along the right edge of the game window, with the game keeping the rest; a game window too narrow for both SHALL show no docked panel.

#### Scenario: Docked by default
- **WHEN** a player starts the game for the first time
- **THEN** the Display panel is docked along the right edge of the game window, and clicks on the rest of the window reach the game as before

#### Scenario: Floating
- **WHEN** the player chooses floating in Options
- **THEN** the docked strip disappears and the Display panel opens as a separate window

### Requirement: Floating Display window
A floating Display panel SHALL be a small separate window that asks to stay above other windows where the platform allows it. Closing it SHALL hide the panel. Closing the game window SHALL close both.

#### Scenario: Closing the floating window
- **WHEN** the player closes the floating Display window
- **THEN** the Display setting becomes hidden, and Options can bring it back

#### Scenario: Closing the game window
- **WHEN** the player closes the game window while the floating Display window is open
- **THEN** both windows close and the game exits

### Requirement: No console window on Windows
On Windows, starting the game normally SHALL open only the game window, with no console window. Starting it with `--terminal` from a console SHALL run in that console.

#### Scenario: Windows launch
- **WHEN** a Windows player starts the game from Steam
- **THEN** only the game window appears

### Requirement: Window frames within budget
Drawing a frame through the window's renderer SHALL fit the performance budget (16 ms per frame at 80x24), measured by a benchmark scenario.

#### Scenario: Window render benchmark
- **WHEN** `just util-bench` runs the window render scenario
- **THEN** its worst frame is reported against the budget like the other scenarios

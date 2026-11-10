# Spec Delta

## Purpose

Playing the desktop game, including on the Steam Deck, with a gamepad instead of a keyboard and mouse: buttons send the keys bound to actions, held directions repeat, and prompts name the buttons.

## ADDED Requirements

### Requirement: Gamepads send bound keys
On desktop builds, the game window SHALL read connected gamepads. Each button and stick direction in the layout SHALL send the first key bound to its action, exactly as pressing that key would, so every screen works and rebinding a key also changes what the button does.

#### Scenario: Moving with the stick
- **WHEN** the player pushes the left stick east on the World page
- **THEN** the robot moves east, exactly as pressing the key bound to `movement.MOVE_EAST` would

#### Scenario: Rebound key
- **WHEN** the player has rebound Mine to another key and presses X
- **THEN** the game receives the rebound key and mines

### Requirement: Default layout
The default layout SHALL be:
- left stick and D-pad: the eight movement directions
- B: Escape
- X: Mine
- Y: cycle build modes
- LB and RB, tapped: previous and next menu tab
- LB, held: the context wheel
- View: Look mode
- Menu: the Menu tab
- left stick click: pass time
- right stick click: torch

The layout SHALL ship in the default configuration.

#### Scenario: Changing tabs
- **WHEN** the player presses RB on the World page
- **THEN** the next menu tab opens, as with the key bound to `ui.MENU_NEXT`

#### Scenario: Diagonal stick
- **WHEN** the player pushes the left stick north-east
- **THEN** the robot moves north-east

### Requirement: A confirms by context
A SHALL send the Interact key on the World page when no window, menu or prompt is open, and Enter everywhere else (windows, menus, prompts, the title screen and other tabs).

#### Scenario: Interacting
- **WHEN** the player stands beside a chest on the World page and presses A
- **THEN** the chest opens, as with the Interact key

#### Scenario: Activating in a window
- **WHEN** a furnace window is open and the player presses A
- **THEN** the selected item moves, as with Enter

### Requirement: Holding LB opens a context wheel
Holding LB past a short delay SHALL open the wheel for the current place: one for the World page in movement mode, one for the build modes, and one for the Inventory tab, each with nothing open. A short tap SHALL keep LB's tab action. Where no wheel exists, LB SHALL act at once.

#### Scenario: Tapping LB
- **WHEN** the player taps LB on the World page
- **THEN** the previous menu tab opens and no wheel is shown

#### Scenario: No wheel here
- **WHEN** a furnace window is open and the player presses LB
- **THEN** LB acts at once, as the key bound to `ui.MENU_PREV` would

### Requirement: Picking from a wheel
While a wheel is open, a stick or D-pad direction SHALL highlight that slot instead of moving. Releasing LB SHALL perform the highlighted slot's action as its button would, or nothing if none is highlighted. B SHALL close the wheel without acting and without its usual effect.

#### Scenario: Picking an action
- **WHEN** the player holds LB on the World page, pushes the right stick north, and releases LB
- **THEN** the wheel's north action (pick up) happens, as with the key bound to `action.PICKUP_ITEMS`, and the robot does not move

#### Scenario: Cancelling
- **WHEN** the player holds LB, highlights a slot and presses B
- **THEN** the wheel closes, nothing happens, and B does not also close anything else

#### Scenario: Releasing on nothing
- **WHEN** the player holds LB and releases it without pushing a direction
- **THEN** the wheel closes and nothing happens

### Requirement: Wheel display
An open wheel SHALL be drawn over the game as a box of eight slots around a centre. Each slot SHALL be named by its action, with the highlighted one marked, and the centre SHALL name the highlighted action or say how to pick one.

#### Scenario: Seeing the wheel
- **WHEN** the player holds LB on the World page
- **THEN** a box shows the eight actions around its centre, with the stick's direction highlighted

### Requirement: Triggers cycle the hotbar
LT and RT SHALL select the previous and next hotbar slot, as one mouse wheel step up or down does on the World page, and SHALL scroll lists elsewhere as the wheel does.

#### Scenario: Next slot
- **WHEN** the player presses RT on the World page
- **THEN** the hotbar selection moves to the next slot

### Requirement: Right stick aims building
In Break, Place and Rotate modes, a right stick direction SHALL send the break or place key for that direction (Break mode sends the break key; Place and Rotate modes send the place key). Outside those modes, the right stick SHALL do nothing.

#### Scenario: Placing a block
- **WHEN** the player is in Place mode with a block selected and flicks the right stick south
- **THEN** the block is placed south of the robot, as with the key bound to `build.PLACE_SOUTH`

#### Scenario: Movement mode
- **WHEN** the player is in movement mode and flicks the right stick
- **THEN** nothing happens

### Requirement: Held directions repeat
A stick or D-pad direction held past a short delay SHALL repeat its key at a steady rate until released or changed. Repeat timing SHALL come from the client clock, so headless runs are deterministic. A stick SHALL count as pushed only past a dead zone.

#### Scenario: Holding east
- **WHEN** the player holds the D-pad east
- **THEN** the robot moves east once at once, then again after the delay, then steadily while it is held

#### Scenario: Resting stick
- **WHEN** the stick drifts slightly from center
- **THEN** no direction is sent

### Requirement: Prompts name the last device
After a gamepad press, every key hint SHALL show the button for actions in the layout ("A: interact", "B: close"). Actions with no button SHALL keep showing their key. After a key press, hints SHALL show keys again.

#### Scenario: Switching to the pad
- **WHEN** the player presses any gamepad button
- **THEN** the context help panel names gamepad buttons, such as "A" for Interact, and says "Hold LB: more actions" where a wheel exists

#### Scenario: Back to the keyboard
- **WHEN** the player then presses a key
- **THEN** the hints show keys again

### Requirement: Gamepad toggle
Options SHALL have a Gamepad row that turns gamepad input on or off. It SHALL be on by default and saved with the other options. While it is off, gamepads SHALL be ignored.

#### Scenario: Turning it off
- **WHEN** the player turns Gamepad off in Options and presses A
- **THEN** nothing happens, and the setting persists after a restart

### Requirement: Desktop only
Mobile builds SHALL NOT read gamepads; they keep the on-screen touch pad. Headless runs SHALL take gamepad input only from script steps.

#### Scenario: Android
- **WHEN** the game runs on a phone
- **THEN** gamepad input is not read, and the touch pad works as before

use crate::config::ConfigRoot;
pub fn default_config() -> ConfigRoot {
    // Match python-old defaults
    let keybinds = serde_json::json!({
        "movement": {
            "MOVE_NORTHWEST": ["NUMPAD_7"],
            "MOVE_NORTH": ["NUMPAD_8"],
            "MOVE_NORTHEAST": ["NUMPAD_9"],
            "MOVE_WEST": ["NUMPAD_4"],
            "WAIT": ["NUMPAD_5"],
            "MOVE_EAST": ["NUMPAD_6"],
            "MOVE_SOUTHWEST": ["NUMPAD_1"],
            "MOVE_SOUTH": ["NUMPAD_2"],
            "MOVE_SOUTHEAST": ["NUMPAD_3"],
            "MOVE_UP": ["<"],
            "MOVE_DOWN": [">"]
        },
        "scale": {"SCALE_UP": ["=", "+"], "SCALE_DOWN": ["-"], "SCALE_RESET": ["0"]},
        "action": {
            "MINE": ["q"],          // Quick mine toggle
            "BUILD": ["e"],         // Quick build toggle
            "INTERACT": ["f"],      // Interact/use
            "PICKUP_ITEMS": ["r"],  // Quick grab
            "LOOK_TOGGLE": ["v"],   // Look around
            "TOGGLE_TORCH": ["t"],  // Toggle torch light source
            "PASS_TIME": [","],     // Let 10 game seconds pass
        },
        "combat": {
            "CLEAR_MOVE_QUEUE": ["c"], // Clear queued moves in combat
        },
        "body": {
            "INSTALL_PART": ["i"], // Body tab: install a fitting part into the selected one (swapping it)
            "REMOVE_PART": ["u"],  // Body tab: take the selected part out
        },
        "build": {
            "TOGGLE_BREAK_PLACE_MODE": ["`"], // Cycle movement/break/place/rotate mode
            "ROTATE_PLACEMENT": ["r"], // Place mode: turn the next factory block clockwise; Blueprint mode: turn the paste
            "BLUEPRINT_MODE": ["b"], // Enter or leave Blueprint mode
            "NEW_BLUEPRINT": ["n"], // Blueprint mode: mark two corners to copy an area
            "BLUEPRINT_LIBRARY": ["B"], // Blueprint mode: open the Blueprints window
            "BREAK_NORTHWEST": ["q"],
            "BREAK_NORTH": ["w"],
            "BREAK_NORTHEAST": ["e"],
            "BREAK_WEST": ["a"],
            "BREAK_CENTER": ["s"],
            "BREAK_EAST": ["d"],
            "BREAK_SOUTHWEST": ["z"],
            "BREAK_SOUTH": ["x"],
            "BREAK_SOUTHEAST": ["c"],
            "PLACE_NORTHWEST": ["q"],
            "PLACE_NORTH": ["w"],
            "PLACE_NORTHEAST": ["e"],
            "PLACE_WEST": ["a"],
            "PLACE_CENTER": ["s"],
            "PLACE_EAST": ["d"],
            "PLACE_SOUTHWEST": ["z"],
            "PLACE_SOUTH": ["x"],
            "PLACE_SOUTHEAST": ["c"]
        },
        "hotbar": {
            "HOTBAR_SLOT_1": ["F1"],
            "HOTBAR_SLOT_2": ["F2"],
            "HOTBAR_SLOT_3": ["F3"],
            "HOTBAR_SLOT_4": ["F4"],
            "HOTBAR_SLOT_5": ["F5"],
            "HOTBAR_SLOT_6": ["F6"],
            "HOTBAR_SLOT_7": ["F7"],
            "HOTBAR_SLOT_8": ["F8"],
            "HOTBAR_SLOT_9": ["F9"],
            "HOTBAR_SLOT_10": ["F10"],
            "HOTBAR_SLOT_11": ["F11"],
            "HOTBAR_SLOT_12": ["F12"]
        },
        "ui": {
            "CLOSE_HELP_MENU": ["ESCAPE"],
            "OPEN_COMMAND_MENU": ["/"],
            "MENU_ACTIVATE": ["ENTER", "SPACE"],
            "MENU_UP": ["UP"],
            "MENU_DOWN": ["DOWN"],
            "MENU_PREV": ["LEFT"],
            "MENU_NEXT": ["RIGHT"],
            "CREDITS_SCROLL_UP": ["UP"],
            "CREDITS_SCROLL_DOWN": ["DOWN"],
            "SAVE_JSON": ["S"],
            "LOAD_JSON": ["L"],
            "TUTORIAL_SELECT": ["T"],
            "TUTORIAL_SKIP": ["Y"],
            "CHEAT_CONSOLE_SCROLL_UP": ["UP"],
            "CHEAT_CONSOLE_SCROLL_DOWN": ["DOWN"],
            "SAVE_AND_QUIT_TO_TITLE": ["Q"], // Menu tab only
            "TOGGLE_CONTEXT_HELP": ["h"], // Context help panel on the World page
            "TAKE_ALL": ["a"], // Chest and corpse windows: take everything
            "QUIT": []
        },
        "title": {
            "UP": ["UP"],
            "DOWN": ["DOWN"],
            "SELECT": ["ENTER"],
            "BACK": ["ESCAPE"],
            "LEFT": ["LEFT"],   // Toggles and volume
            "RIGHT": ["RIGHT"],
            "DELETE_CHAR": ["BACKSPACE"] // Text fields
        },
        "mouse": {
            "PLACE": ["LEFT_CLICK"],         // On the world: open what's there, else place the hotbar block, else look
            "BREAK": ["RIGHT_CLICK"],        // On the world: mine or break
            "PICK_BLOCK": ["MIDDLE_CLICK"],  // Select the clicked block's item and copy its facing
            "ROTATE": ["SHIFT+SCROLL_UP", "SHIFT+SCROLL_DOWN"], // Turn the next placement
            "NEXT_SLOT": ["SCROLL_DOWN"],    // Next hotbar slot
            "PREV_SLOT": ["SCROLL_UP"]       // Previous hotbar slot
        },
        "blueprints": {
            "RENAME": ["r"],              // Blueprints window: rename the selected blueprint
            "DELETE": ["x"],              // Blueprints window: delete the selected blueprint
            "DELETE_CHAR": ["BACKSPACE"], // While renaming: erase a character
            "END_RENAME": ["ENTER"]       // While renaming: keep the new name
        },
        "global_map": {
            "DELETE_MARKER": ["x"],              // Global Map: delete the selected wreck marker
            "CONFIRM_DELETE": ["y"],             // Delete marker prompt: yes
            "CANCEL_DELETE": ["n", "ESCAPE"]     // Delete marker prompt: no
        },
        "crafting": {
            "TOGGLE_CRAFTABLE_ONLY": ["c"], // Crafting tab: list only what you can craft now
        },
        "search": {
            "SEARCH": ["s"],              // Item Dictionary and Crafting: start typing a search
            "DELETE_CHAR": ["BACKSPACE"], // While searching: erase a character
            "END_SEARCH": ["ENTER"]       // While searching: keep the filter and stop typing
        },
        "inventory": {
            "DROP_ITEM": ["g"], // Drop item
            "DESTROY_ITEM": ["x"], // Destroy item, remove later
            "CHEAT_DUPLICATE_ITEM": ["."], // Duplicate item, remove later
            "TOGGLE_ITEM_AUTO_PICKUP_KEY": ["p"] // Toggle item auto pickup
        }
    });
    let settings = serde_json::json!({
        "game": {
            "GAME_NAME": "LithicRivers",
            "LOGFILENAME": "LithicRivers.log",
            "LOGGINGLEVEL": "INFO",
            "SAVES_FOLDER": "lithicrivers-saves/saves",
            "SNAPSHOTS_FOLDER": "lithicrivers-saves/snapshots",
            "DEVELOPER_MODE": true,
            "DEFAULT_SEED": 4669201609u64,
            "DEFAULT_PLAYER_NAME": "melty",
            "COMBAT_MS_PER_TICK": 1000u64,
            "METERS_PER_BLOCK": 1u64,
        },
        "inventory": {
            "TOGGLE_ITEM_AUTO_PICKUP_DEFAULT_ENABLED": true
        },
        "ui": {
            "SHOW_CONTEXT_HELP": true
        },
        "audio": {
            "MUSIC_VOLUME": 0.60,
            "SFX_VOLUME": 0.35
        },
        "world": {
            "DEFAULT_PLAYER_POSITION": {"production": [25, 25, 0], "testing": [0, 0, 0]}
        },
        "performance": {"MAX_CPU_THREADS": 64}
    });
    ConfigRoot { keybinds, settings }
}

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
        },
        "combat": {
            "CLEAR_MOVE_QUEUE": ["c"], // Clear queued moves in combat
        },
        "build": {
            "TOGGLE_BREAK_PLACE_MODE": ["`"], // Cycle break/place mode
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
            "MENU_PREV": ["LEFT"],
            "MENU_NEXT": ["RIGHT"],
            "CREDITS_SCROLL_UP": ["UP"],
            "CREDITS_SCROLL_DOWN": ["DOWN"],
            "SAVE_JSON": ["S"],
            "LOAD_JSON": ["L"],
            "TUTORIAL_SELECT": ["T"],
            "TUTORIAL_SKIP": ["Y"],
            "QUIT": []
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
        "world": {
            "DEFAULT_PLAYER_POSITION": {"production": [25, 25, 0], "testing": [0, 0, 0]}
        },
        "performance": {"MAX_CPU_THREADS": 64}
    });
    ConfigRoot { keybinds, settings }
}

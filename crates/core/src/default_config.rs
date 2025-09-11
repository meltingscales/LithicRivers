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
            "LOOK_TOGGLE": ["c"],   // Look around
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
            "COMBAT_MS_PER_TICK": 1000u64
        },
        "inventory": {
            "TOGGLE_ITEM_AUTO_PICKUP_DEFAULT_ENABLED": true
        },
        "world": {
            "DEFAULT_SIZE_RADIUS": {"production": [50, 50, 3], "testing": [3, 3, 1]},
            "DEFAULT_PLAYER_POSITION": {"production": [25, 25, 0], "testing": [0, 0, 0]}
        },
        "performance": {"MAX_CPU_THREADS": 64}
    });
    ConfigRoot { keybinds, settings }
}

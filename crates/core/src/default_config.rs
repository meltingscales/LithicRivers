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
        "viewport": {
            "RESET_VIEWPORT": ["r"],
            "SLIDE_VIEWPORT_WEST": ["["],
            "SLIDE_VIEWPORT_EAST": ["]"],
            "TOGGLE_VIEWPORT": ["v"],
            "VIEW_Z_UP": ["PAGEUP"],
            "VIEW_Z_DOWN": ["PAGEDOWN"]
        },
        "scale": {"SCALE_UP": ["=", "+"], "SCALE_DOWN": ["-"], "SCALE_RESET": ["0"]},
        "action": {"MINE": ["u"], "INTERACT": ["i"], "PICKUP_ITEMS": ["g"]},
        "ui": {
            "CLOSE_HELP_MENU": ["ESCAPE"],
            "OPEN_COMMAND_MENU": ["/"],
            "MENU_ACTIVATE": ["ENTER", "SPACE"],
            "MENU_PREV": ["LEFT"],
            "MENU_NEXT": ["RIGHT"],
            "CREDITS_SCROLL_UP": ["UP"],
            "CREDITS_SCROLL_DOWN": ["DOWN"],
            "LOOK_TOGGLE": ["l"],
            "SAVE_JSON": ["S"],
            "LOAD_JSON": ["L"],
            "QUIT": []
        },
        "inventory": {"DROP_ITEM": ["d"], "DESTROY_ITEM": ["x"], "CHEAT_DUPLICATE_ITEM": ["."]}
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
            "DEFAULT_PLAYER_NAME": "melty"
        },
        "world": {
            "DEFAULT_SIZE_RADIUS": {"production": [50, 50, 3], "testing": [3, 3, 1]},
            "DEFAULT_PLAYER_POSITION": {"production": [25, 25, 0], "testing": [0, 0, 0]}
        },
        "performance": {"MAX_CPU_THREADS": 64}
    });
    ConfigRoot { keybinds, settings }
}

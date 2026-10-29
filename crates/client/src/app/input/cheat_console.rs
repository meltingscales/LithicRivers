use crate::app_state::CheatConsoleState;
use crate::App;
use crossterm::event::KeyCode;
use lithicrivers_core::components::Position;
use std::collections::HashMap;
use std::error::Error;

/// Handler invoked with the app and the command's arguments
pub type CommandHandler = fn(&mut App, &[&str]) -> Result<(), Box<dyn Error>>;

/// Command descriptor with name, description, and handler
pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
    pub handler: CommandHandler,
}

/// Registry of all available commands
/// Commands the console's help list shows at once; it scrolls through the rest.
pub const HELP_COMMAND_ROWS: usize = 5;

pub struct CommandRegistry {
    commands: HashMap<&'static str, Command>,
}

impl CommandRegistry {
    /// Create a new command registry with all available commands
    pub fn new() -> Self {
        let mut commands = HashMap::new();

        // Register all commands
        let cmd_list = [
            Command {
                name: "tp",
                description: "tp x y z - Teleport to coordinates (~ = relative to player)",
                handler: handle_tp_command,
            },
            Command {
                name: "tp_quest",
                description: "tp_quest [android|facility|locker] - Teleport to a quest spot (none = the next one)",
                handler: handle_tp_quest_command,
            },
            Command {
                name: "noclip_toggle",
                description: "noclip_toggle - Toggle noclip mode",
                handler: handle_noclip_toggle_command,
            },
            Command {
                name: "toggle_fogofwar",
                description: "toggle_fogofwar - Toggle fog of war rendering",
                handler: handle_toggle_fogofwar_command,
            },
            Command {
                name: "set_time",
                description: "set_time hh:mm - Skip forward to the next time the clock reads hh:mm",
                handler: handle_set_time_command,
            },
            Command {
                name: "toggle_tutorial",
                description: "toggle_tutorial - Toggle tutorial panel visibility",
                handler: handle_toggle_tutorial_command,
            },
            Command {
                name: "kill_player",
                description: "kill_player - Destroy your body (to test dying and respawning)",
                handler: handle_kill_player_command,
            },
            Command {
                name: "suicide",
                description: "suicide - Destroy your body (the same as kill_player)",
                handler: handle_kill_player_command,
            },
            Command {
                name: "keepinventory",
                description: "keepinventory [on|off] - Keep everything you carry when you die",
                handler: handle_keepinventory_command,
            },
            Command {
                name: "keepbody",
                description: "keepbody [on|off] - Respawn in your old body, repaired",
                handler: handle_keepbody_command,
            },
            Command {
                name: "set_tile",
                description: "set_tile x y z <tile> - Replace a tile, e.g. set_tile ~1 ~ ~ tree (~ = relative to player)",
                handler: handle_set_tile_command,
            },
            Command {
                name: "spawn_enemy",
                description:
                    "spawn_enemy <kind> [x y z] - Spawn an enemy (default ~2 ~ ~, ~ = relative to player)",
                handler: handle_spawn_enemy_command,
            },
            Command {
                name: "giveitem",
                description:
                    "giveitem <item_sprite_name> [quantity] - Give items to player inventory (1 if no quantity)",
                handler: handle_giveitem_command,
            },
            Command {
                name: "givecraftingcomponents",
                description:
                    "givecraftingcomponents <item> [amount] [raw] - Give what crafting the item needs (raw: its raw materials)",
                handler: handle_givecraftingcomponents_command,
            },
        ];

        for cmd in cmd_list.iter() {
            commands.insert(
                cmd.name,
                Command {
                    name: cmd.name,
                    description: cmd.description,
                    handler: cmd.handler,
                },
            );
        }

        Self { commands }
    }

    /// Get commands that match a prefix for autocomplete
    pub fn get_matching_commands(&self, prefix: &str) -> Vec<&'static str> {
        let mut matches: Vec<_> = self
            .commands
            .keys()
            .filter(|name| name.starts_with(prefix))
            .copied()
            .collect();
        matches.sort();
        matches
    }

    /// Get command descriptions for help
    pub fn get_command_descriptions(&self) -> Vec<&'static str> {
        let mut descriptions: Vec<_> = self.commands.values().map(|cmd| cmd.description).collect();
        descriptions.sort();
        descriptions
    }

    /// Execute a command
    pub fn execute(&self, app: &mut App, command: &str) -> Result<(), Box<dyn Error>> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        if let Some(cmd) = self.commands.get(parts[0]) {
            (cmd.handler)(app, &parts)
        } else {
            // Unknown command - could add error message system later
            Ok(())
        }
    }
}

/// Get the global command registry
pub fn get_command_registry() -> CommandRegistry {
    CommandRegistry::new()
}

/// Find the longest common prefix among a list of strings
fn find_common_prefix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }

    if strings.len() == 1 {
        return strings[0].clone();
    }

    let first = &strings[0];
    let mut common_len = 0;

    for (i, ch) in first.chars().enumerate() {
        if strings.iter().all(|s| s.chars().nth(i) == Some(ch)) {
            common_len = i + 1;
        } else {
            break;
        }
    }

    first.chars().take(common_len).collect()
}

/// Update autocomplete suggestions based on current input
fn update_autocomplete(input: &str) -> (Vec<String>, Option<usize>) {
    if input.is_empty() {
        return (Vec::new(), None);
    }

    let parts: Vec<&str> = input.split_whitespace().collect();

    if parts.is_empty() {
        return (Vec::new(), None);
    }

    // If we're still typing the command name (no spaces or just started)
    if parts.len() == 1 && !input.ends_with(' ') {
        let registry = get_command_registry();
        let matches: Vec<String> = registry
            .get_matching_commands(input)
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        return (matches, None);
    }

    // Handle sub-command autocomplete
    let command_name = parts[0];
    match command_name {
        "giveitem" => {
            if parts.len() == 1 || (parts.len() == 2 && !input.ends_with(' ')) {
                // Autocomplete item names for first argument
                let prefix = if parts.len() == 2 { parts[1] } else { "" };
                let item_names = lithicrivers_core::components::get_all_item_sprite_names();
                let matches: Vec<String> = item_names
                    .into_iter()
                    .filter(|name| name.starts_with(prefix))
                    .map(|name| format!("giveitem {}", name))
                    .collect();
                return (matches, None);
            } else if parts.len() == 2 || (parts.len() == 3 && !input.ends_with(' ')) {
                // Show quantity suggestions for second argument
                let base = format!("giveitem {}", parts[1]);
                let quantity_suggestions = vec!["1", "5", "10", "50", "100"];
                let prefix = if parts.len() == 3 { parts[2] } else { "" };
                let matches: Vec<String> = quantity_suggestions
                    .into_iter()
                    .filter(|qty| qty.starts_with(prefix))
                    .map(|qty| format!("{} {}", base, qty))
                    .collect();
                return (matches, None);
            }
        }
        "givecraftingcomponents"
            if parts.len() == 1 || (parts.len() == 2 && !input.ends_with(' ')) =>
        {
            // Only items that have a recipe
            let prefix = parts.get(1).copied().unwrap_or("");
            let recipes = lithicrivers_core::recipe_handler::RecipeHandler::new();
            let mut names: Vec<&str> = recipes
                .get_recipes()
                .iter()
                .map(|r| lithicrivers_core::components::itemkind_sprite_name(r.result))
                .filter(|name| name.starts_with(prefix))
                .collect();
            names.sort();
            names.dedup();
            let matches = names
                .into_iter()
                .map(|name| format!("givecraftingcomponents {name}"))
                .collect();
            return (matches, None);
        }
        "keepinventory" | "keepbody"
            if parts.len() == 1 || (parts.len() == 2 && !input.ends_with(' ')) =>
        {
            let prefix = parts.get(1).copied().unwrap_or("");
            let matches = ["on", "off"]
                .into_iter()
                .filter(|state| state.starts_with(prefix))
                .map(|state| format!("{command_name} {state}"))
                .collect();
            return (matches, None);
        }
        "tp_quest" if parts.len() == 1 || (parts.len() == 2 && !input.ends_with(' ')) => {
            let prefix = parts.get(1).copied().unwrap_or("");
            let matches = lithicrivers_core::quests::TP_QUEST_TARGETS
                .into_iter()
                .filter(|t| t.starts_with(prefix))
                .map(|t| format!("tp_quest {t}"))
                .collect();
            return (matches, None);
        }
        "tp" if parts.len() < 4 => {
            // For tp command, we could suggest coordinate patterns
            let coord_suggestions = vec!["0 0 0", "10 10 0", "100 100 0"];
            let matches: Vec<String> = coord_suggestions
                .into_iter()
                .map(|coords| format!("tp {}", coords))
                .collect();
            return (matches, None);
        }
        _ => {
            // No sub-command autocomplete for other commands
        }
    }

    (Vec::new(), None)
}

// Individual command handlers
/// Parse three coordinates, each absolute (`12`) or relative to `player` (`~`, `~3`, `~-2`).
fn parse_position(args: &[&str], player: Position) -> Result<Position, String> {
    let [x, y, z] = args else {
        return Err(format!("expected 3 coordinates, got {}", args.len()));
    };
    let axis = |arg: &str, player_axis: i64| -> Result<i64, String> {
        let value = match arg.strip_prefix('~') {
            Some("") => Ok(player_axis),
            Some(offset) => offset.parse::<i64>().map(|o| player_axis + o),
            None => arg.parse::<i64>(),
        };
        value.map_err(|_| format!("invalid coordinate '{arg}' (use a number, ~ or ~<offset>)"))
    };
    Ok(Position {
        x: axis(x, player.x)?,
        y: axis(y, player.y)?,
        z: axis(z, player.z)?,
    })
}

fn player_position(app: &App) -> Result<Position, Box<dyn Error>> {
    app.core
        .game
        .get_player_position()
        .ok_or_else(|| "no player in the world".into())
}

fn handle_tp_command(app: &mut App, parts: &[&str]) -> Result<(), Box<dyn Error>> {
    let player = player_position(app)?;
    let new_position =
        parse_position(&parts[1..], player).map_err(|e| format!("Usage: tp x y z - {e}"))?;
    teleport_player(app, new_position);
    Ok(())
}

fn handle_tp_quest_command(app: &mut App, parts: &[&str]) -> Result<(), Box<dyn Error>> {
    let game = &mut app.core.game;
    let pos = lithicrivers_core::quests::tp_quest_target(
        &game.world,
        &mut game.res,
        parts.get(1).copied(),
    )?;
    teleport_player(app, pos);
    app.core
        .game
        .res
        .log(format!("Teleported to ({}, {}, {})", pos.x, pos.y, pos.z));
    Ok(())
}

/// Move the player, with the view and look cursor following.
fn teleport_player(app: &mut App, new_position: Position) {
    if let Some(player_entity) = app.core.game.get_player_entity() {
        if let Ok(mut pos) = app.core.game.world.get::<&mut Position>(player_entity) {
            *pos = new_position;
        }
    }
    // Load the new place now (chunks, structures, explored map, fog of war), not on the
    // first step there
    app.core.game.load_around_player();
    // Update UI viewport and look cursor to follow player
    app.ui.view_x = new_position.x;
    app.ui.view_y = new_position.y;
    app.ui.view_z = new_position.z;
    app.panels.look.cursor = new_position;
}

fn handle_set_tile_command(app: &mut App, parts: &[&str]) -> Result<(), Box<dyn Error>> {
    use lithicrivers_core::tile_registry::TileKind;
    let usage = || {
        format!(
            "Usage: set_tile x y z <tile>; tiles: {}",
            TileKind::all_keys().join(", ")
        )
    };
    let [_, x, y, z, name] = parts else {
        return Err(usage().into());
    };
    let tile =
        TileKind::from_key(name).ok_or_else(|| format!("unknown tile '{name}' - {}", usage()))?;
    let player = player_position(app)?;
    let pos = parse_position(&[x, y, z], player).map_err(|e| format!("{} - {e}", usage()))?;
    let game = &mut app.core.game;
    game.res
        .world_state
        .world
        .set_tile_cached(pos.x, pos.y, pos.z, tile);
    game.res.log(format!(
        "Set ({}, {}, {}) to {}",
        pos.x,
        pos.y,
        pos.z,
        tile.display_name()
    ));
    Ok(())
}

fn handle_spawn_enemy_command(app: &mut App, parts: &[&str]) -> Result<(), Box<dyn Error>> {
    use lithicrivers_core::spawn_utils::{spawn_enemy, EnemyKind};

    let kinds = || {
        EnemyKind::ALL
            .iter()
            .map(|k| k.name())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let usage = || format!("Usage: spawn_enemy <kind> [x y z]; kinds: {}", kinds());
    let Some(name) = parts.get(1) else {
        return Err(usage().into());
    };
    let kind = EnemyKind::from_name(name)
        .ok_or_else(|| format!("unknown enemy '{name}'; kinds: {}", kinds()))?;
    let player = player_position(app)?;
    let coords: &[&str] = match &parts[2..] {
        [] => &["~2", "~", "~"],
        rest => rest,
    };
    let pos = parse_position(coords, player).map_err(|e| format!("{} - {e}", usage()))?;

    let game = &mut app.core.game;
    spawn_enemy(&mut game.world, &mut game.res.world_state.world, kind, pos).map_err(|e| {
        format!(
            "can't spawn {} at ({}, {}, {}): {e}",
            kind.name(),
            pos.x,
            pos.y,
            pos.z
        )
    })?;
    game.res.log(format!(
        "Spawned {} at ({}, {}, {})",
        kind.name(),
        pos.x,
        pos.y,
        pos.z
    ));
    Ok(())
}

/// Skip forward to the next moment the clock reads `hh:mm`, never backward, so tick-based
/// timers stay in order. Factories and plants catch up on the next tick.
fn handle_set_time_command(app: &mut App, parts: &[&str]) -> Result<(), Box<dyn Error>> {
    let usage = "Usage: set_time hh:mm (00:00 to 23:59)";
    let (hour, minute) = parts
        .get(1)
        .and_then(|t| t.split_once(':'))
        .and_then(|(h, m)| Some((h.parse::<u64>().ok()?, m.parse::<u64>().ok()?)))
        .filter(|&(h, m)| h < 24 && m < 60)
        .ok_or(usage)?;
    let time = &mut app.core.game.res.time;
    time.tick += time.ticks_until(hour, minute);
    let (h, m) = time.clock();
    app.core
        .game
        .res
        .log(format!("The time is now {h:02}:{m:02}"));
    Ok(())
}

fn handle_noclip_toggle_command(app: &mut App, _parts: &[&str]) -> Result<(), Box<dyn Error>> {
    app.core.game.res.player_state.noclip_enabled = !app.core.game.res.player_state.noclip_enabled;
    let state = if app.core.game.res.player_state.noclip_enabled {
        "enabled"
    } else {
        "disabled"
    };
    app.core.game.res.log(format!("Noclip mode: {}", state));
    Ok(())
}

fn handle_toggle_fogofwar_command(app: &mut App, _parts: &[&str]) -> Result<(), Box<dyn Error>> {
    app.core.game.res.player_state.fog_of_war_enabled =
        !app.core.game.res.player_state.fog_of_war_enabled;
    let state = if app.core.game.res.player_state.fog_of_war_enabled {
        "enabled"
    } else {
        "disabled"
    };
    app.core.game.res.log(format!("Fog of war: {}", state));
    Ok(())
}

fn handle_kill_player_command(app: &mut App, _parts: &[&str]) -> Result<(), Box<dyn Error>> {
    let game = &mut app.core.game;
    let player = game.get_player_entity().ok_or("no player")?;
    if lithicrivers_core::death::is_dead(&game.world, player) {
        return Err("you are already dead".into());
    }
    lithicrivers_core::death::player_died(
        &mut game.world,
        &mut game.res,
        player,
        "self-destruct command".to_string(),
    );
    Ok(())
}

fn handle_keepinventory_command(app: &mut App, parts: &[&str]) -> Result<(), Box<dyn Error>> {
    let settings = &mut app.core.game.res.world_settings;
    let state = switch(&mut settings.keep_inventory, parts, "keepinventory")?;
    app.core.game.res.log(format!("Keep inventory: {state}"));
    Ok(())
}

fn handle_keepbody_command(app: &mut App, parts: &[&str]) -> Result<(), Box<dyn Error>> {
    let settings = &mut app.core.game.res.world_settings;
    let state = switch(&mut settings.keep_body, parts, "keepbody")?;
    app.core.game.res.log(format!("Keep body: {state}"));
    Ok(())
}

/// Set an on/off world setting from `parts[1]` (or leave it, without an argument) and
/// return its state as "on" or "off".
fn switch(setting: &mut bool, parts: &[&str], name: &str) -> Result<&'static str, String> {
    match parts.get(1).copied() {
        None => {}
        Some("on") => *setting = true,
        Some("off") => *setting = false,
        Some(_) => return Err(format!("Usage: {name} [on|off]")),
    }
    Ok(if *setting { "on" } else { "off" })
}

fn handle_toggle_tutorial_command(app: &mut App, _parts: &[&str]) -> Result<(), Box<dyn Error>> {
    use crate::MenuTab;

    // Toggle tutorial visibility
    app.ui.tutorial_visible = !app.ui.tutorial_visible;
    app.core.game.res.world_settings.show_tutorial = app.ui.tutorial_visible;

    if app.ui.tutorial_visible {
        // Tutorial is now visible - switch to Tutorial tab
        app.ui.current_tab = MenuTab::Tutorial;
        app.core.game.res.log("Tutorial system enabled");
    } else {
        // Tutorial is now hidden - switch away from Tutorial tab if currently on it
        if app.ui.current_tab == MenuTab::Tutorial {
            app.ui.current_tab = MenuTab::World;
        }
        app.core.game.res.log("Tutorial system disabled");
    }
    Ok(())
}

fn handle_givecraftingcomponents_command(
    app: &mut App,
    parts: &[&str],
) -> Result<(), Box<dyn Error>> {
    use lithicrivers_core::components::{itemkind_name, parse_itemkind_from_sprite_name};
    let usage = "Usage: givecraftingcomponents <item> [amount] [raw]";
    let Some(name) = parts.get(1) else {
        return Err(usage.into());
    };
    let item = parse_itemkind_from_sprite_name(name).ok_or_else(|| {
        format!(
            "Unknown item '{name}'. Items: {}",
            lithicrivers_core::components::get_all_item_sprite_names().join(", ")
        )
    })?;
    let (mut amount, mut raw) = (1u32, false);
    for arg in &parts[2..] {
        match (*arg, arg.parse::<u32>()) {
            ("raw", _) => raw = true,
            (_, Ok(n)) if n > 0 => amount = n,
            _ => return Err(format!("Unexpected argument '{arg}'. {usage}").into()),
        }
    }
    let recipes = lithicrivers_core::recipe_handler::RecipeHandler::new();
    let components =
        lithicrivers_core::crafting::components_for(recipes.get_recipes(), item, amount, raw)
            .map_err(|e| e.to_string())?;
    let Some(player) = app.core.game.get_player_entity() else {
        return Err("No player".into());
    };
    if let Ok(mut inv) = app
        .core
        .game
        .world
        .get::<&mut lithicrivers_core::components::Inventory>(player)
    {
        for &(kind, qty) in &components {
            inv.add(kind, qty);
        }
    }
    let given: Vec<String> = components
        .iter()
        .map(|(kind, qty)| format!("{qty} {}", itemkind_name(*kind)))
        .collect();
    app.core.game.res.log(format!(
        "Gave {} for {amount} {}",
        given.join(", "),
        itemkind_name(item)
    ));
    Ok(())
}

fn handle_giveitem_command(app: &mut App, parts: &[&str]) -> Result<(), Box<dyn Error>> {
    if parts.len() < 2 {
        app.core
            .game
            .res
            .log("Usage: giveitem <item_sprite_name> [quantity, default 1]".to_string());
        let available_items = lithicrivers_core::components::get_all_item_sprite_names().join(", ");
        app.core
            .game
            .res
            .log(format!("Available items: {}", available_items));
        return Ok(());
    }

    // Parse quantity (1 when left out)
    let quantity = match parts.get(2).map_or(Ok(1), |q| q.parse::<u32>()) {
        Ok(q) if q > 0 => q,
        _ => {
            app.core
                .game
                .res
                .log("Error: Quantity must be a positive number".to_string());
            return Ok(());
        }
    };

    // Parse item using sprite name
    let item_kind = match lithicrivers_core::components::parse_itemkind_from_sprite_name(parts[1]) {
        Some(kind) => kind,
        None => {
            app.core
                .game
                .res
                .log(format!("Error: Unknown item '{}'", parts[1]));
            let available_items =
                lithicrivers_core::components::get_all_item_sprite_names().join(", ");
            app.core
                .game
                .res
                .log(format!("Available items: {}", available_items));
            return Ok(());
        }
    };

    // Get player entity and add item to inventory
    if let Some(player_entity) = app.core.game.get_player_entity() {
        // First, add the item to inventory
        let add_success = if let Ok(mut inventory) =
            app.core
                .game
                .world
                .get::<&mut lithicrivers_core::components::Inventory>(player_entity)
        {
            inventory.add(item_kind, quantity);
            let item_display_name = lithicrivers_core::components::itemkind_name(item_kind);
            app.core.game.res.log(format!(
                "Added {} {} to inventory",
                quantity, item_display_name
            ));
            true
        } else {
            app.core
                .game
                .res
                .log("Error: Player has no inventory!".to_string());
            false
        };

        // Then, if successful, update quest objectives (separate borrow)
        if add_success {
            if let Ok(player_inventory) =
                app.core
                    .game
                    .world
                    .get::<&lithicrivers_core::components::Inventory>(player_entity)
            {
                app.core
                    .game
                    .res
                    .update_quest_objectives_from_inventory(&player_inventory);
            }
        }
    } else {
        app.core
            .game
            .res
            .log("Error: Player not found!".to_string());
    }

    Ok(())
}

pub fn handle_input(app: &mut App, key: KeyCode) -> Result<bool, Box<dyn Error>> {
    match &app.panels.cheat_console {
        CheatConsoleState::None => {
            // Check if we should open the console with OPEN_COMMAND_MENU keybind
            if app.ui.keybinds.matches("ui", "OPEN_COMMAND_MENU", &key) {
                if !app.core.game.res.world_settings.cheats_allowed {
                    app.core
                        .game
                        .res
                        .log_yellow("Cheats are disabled for this world");
                    return Ok(true);
                }
                app.panels.cheat_console = CheatConsoleState::Open {
                    input: String::new(),
                    cursor_position: 0,
                    autocomplete_suggestions: Vec::new(),
                    autocomplete_index: None,
                    scroll_offset: 0,
                };
                return Ok(true);
            }
        }
        CheatConsoleState::Open {
            input,
            cursor_position,
            autocomplete_suggestions,
            autocomplete_index,
            scroll_offset,
        } => {
            match key {
                // Close console with Escape
                KeyCode::Esc => {
                    app.panels.cheat_console = CheatConsoleState::None;
                    return Ok(true);
                }
                // Enter on a picked suggestion fills it in; otherwise it runs the input
                KeyCode::Enter => {
                    let picked = autocomplete_index
                        .and_then(|i| autocomplete_suggestions.get(i))
                        .filter(|s| *s != input)
                        .cloned();
                    if let Some(picked) = picked {
                        let (suggestions, index) = update_autocomplete(&picked);
                        app.panels.cheat_console = CheatConsoleState::Open {
                            cursor_position: picked.len(),
                            input: picked,
                            autocomplete_suggestions: suggestions,
                            autocomplete_index: index,
                            scroll_offset: *scroll_offset,
                        };
                        return Ok(true);
                    }
                    let command = input.clone();
                    app.panels.cheat_console = CheatConsoleState::None;
                    execute_command(app, &command);
                    return Ok(true);
                }
                // Handle text input
                KeyCode::Char(c) => {
                    let mut new_input = input.clone();
                    new_input.insert(*cursor_position, c);
                    let new_cursor = cursor_position + 1;
                    let (suggestions, index) = update_autocomplete(&new_input);
                    app.panels.cheat_console = CheatConsoleState::Open {
                        input: new_input,
                        cursor_position: new_cursor,
                        autocomplete_suggestions: suggestions,
                        autocomplete_index: index,
                        scroll_offset: *scroll_offset,
                    };
                    return Ok(true);
                }
                // Handle backspace
                KeyCode::Backspace => {
                    if *cursor_position > 0 {
                        let mut new_input = input.clone();
                        new_input.remove(*cursor_position - 1);
                        let new_cursor = cursor_position - 1;
                        let (suggestions, index) = update_autocomplete(&new_input);
                        app.panels.cheat_console = CheatConsoleState::Open {
                            input: new_input,
                            cursor_position: new_cursor,
                            autocomplete_suggestions: suggestions,
                            autocomplete_index: index,
                            scroll_offset: *scroll_offset,
                        };
                    }
                    return Ok(true);
                }
                // Handle cursor movement
                KeyCode::Left => {
                    if *cursor_position > 0 {
                        app.panels.cheat_console = CheatConsoleState::Open {
                            input: input.clone(),
                            cursor_position: cursor_position - 1,
                            autocomplete_suggestions: autocomplete_suggestions.clone(),
                            autocomplete_index: *autocomplete_index,
                            scroll_offset: *scroll_offset,
                        };
                    }
                    return Ok(true);
                }
                KeyCode::Right => {
                    if *cursor_position < input.len() {
                        app.panels.cheat_console = CheatConsoleState::Open {
                            input: input.clone(),
                            cursor_position: cursor_position + 1,
                            autocomplete_suggestions: autocomplete_suggestions.clone(),
                            autocomplete_index: *autocomplete_index,
                            scroll_offset: *scroll_offset,
                        };
                    }
                    return Ok(true);
                }
                // Home/End keys
                KeyCode::Home => {
                    app.panels.cheat_console = CheatConsoleState::Open {
                        input: input.clone(),
                        cursor_position: 0,
                        autocomplete_suggestions: autocomplete_suggestions.clone(),
                        autocomplete_index: *autocomplete_index,
                        scroll_offset: *scroll_offset,
                    };
                    return Ok(true);
                }
                KeyCode::End => {
                    app.panels.cheat_console = CheatConsoleState::Open {
                        input: input.clone(),
                        cursor_position: input.len(),
                        autocomplete_suggestions: autocomplete_suggestions.clone(),
                        autocomplete_index: *autocomplete_index,
                        scroll_offset: *scroll_offset,
                    };
                    return Ok(true);
                }
                // Delete key
                KeyCode::Delete => {
                    if *cursor_position < input.len() {
                        let mut new_input = input.clone();
                        new_input.remove(*cursor_position);
                        let (suggestions, index) = update_autocomplete(&new_input);
                        app.panels.cheat_console = CheatConsoleState::Open {
                            input: new_input,
                            cursor_position: *cursor_position,
                            autocomplete_suggestions: suggestions,
                            autocomplete_index: index,
                            scroll_offset: *scroll_offset,
                        };
                    }
                    return Ok(true);
                }
                // Tab for autocomplete - bash-like completion
                KeyCode::Tab => {
                    if !autocomplete_suggestions.is_empty() {
                        if autocomplete_suggestions.len() == 1 {
                            // Single match - complete it fully
                            let completed = autocomplete_suggestions[0].clone();
                            let (new_suggestions, new_index) = update_autocomplete(&completed);
                            app.panels.cheat_console = CheatConsoleState::Open {
                                input: completed.clone(),
                                cursor_position: completed.len(),
                                autocomplete_suggestions: new_suggestions,
                                autocomplete_index: new_index,
                                scroll_offset: *scroll_offset,
                            };
                        } else {
                            // Multiple matches - bash-like behavior
                            let common_prefix = find_common_prefix(autocomplete_suggestions);

                            // If we can extend the input with the common prefix, do that first
                            if common_prefix.len() > input.len() {
                                let (new_suggestions, new_index) =
                                    update_autocomplete(&common_prefix);
                                app.panels.cheat_console = CheatConsoleState::Open {
                                    input: common_prefix.clone(),
                                    cursor_position: common_prefix.len(),
                                    autocomplete_suggestions: new_suggestions,
                                    autocomplete_index: new_index,
                                    scroll_offset: *scroll_offset,
                                };
                            } else {
                                // No more common prefix to complete - cycle through suggestions
                                let new_index = match autocomplete_index {
                                    Some(idx) => Some((idx + 1) % autocomplete_suggestions.len()),
                                    None => Some(0),
                                };
                                app.panels.cheat_console = CheatConsoleState::Open {
                                    input: input.clone(),
                                    cursor_position: *cursor_position,
                                    autocomplete_suggestions: autocomplete_suggestions.clone(),
                                    autocomplete_index: new_index,
                                    scroll_offset: *scroll_offset,
                                };
                            }
                        }
                    }
                    return Ok(true);
                }
                // Handle scrolling through command list
                key if app
                    .ui
                    .keybinds
                    .matches("ui", "CHEAT_CONSOLE_SCROLL_UP", &key) =>
                {
                    if !autocomplete_suggestions.is_empty() {
                        // Pick the previous suggestion
                        app.panels.cheat_console = CheatConsoleState::Open {
                            input: input.clone(),
                            cursor_position: *cursor_position,
                            autocomplete_suggestions: autocomplete_suggestions.clone(),
                            autocomplete_index: Some(
                                autocomplete_index.unwrap_or(0).saturating_sub(1),
                            ),
                            scroll_offset: *scroll_offset,
                        };
                    } else if input.is_empty() {
                        // Only scroll when showing all commands (no input/autocomplete)
                        let new_scroll_offset = scroll_offset.saturating_sub(1);
                        app.panels.cheat_console = CheatConsoleState::Open {
                            input: input.clone(),
                            cursor_position: *cursor_position,
                            autocomplete_suggestions: autocomplete_suggestions.clone(),
                            autocomplete_index: *autocomplete_index,
                            scroll_offset: new_scroll_offset,
                        };
                    }
                    return Ok(true);
                }
                key if app
                    .ui
                    .keybinds
                    .matches("ui", "CHEAT_CONSOLE_SCROLL_DOWN", &key) =>
                {
                    if !autocomplete_suggestions.is_empty() {
                        // Pick the next suggestion
                        let last = autocomplete_suggestions.len() - 1;
                        let next = autocomplete_index.map_or(0, |i| (i + 1).min(last));
                        app.panels.cheat_console = CheatConsoleState::Open {
                            input: input.clone(),
                            cursor_position: *cursor_position,
                            autocomplete_suggestions: autocomplete_suggestions.clone(),
                            autocomplete_index: Some(next),
                            scroll_offset: *scroll_offset,
                        };
                    } else if input.is_empty() {
                        // Only scroll when showing all commands (no input/autocomplete)
                        let registry = get_command_registry();
                        let commands = registry.get_command_descriptions();
                        let max_scroll = commands.len().saturating_sub(HELP_COMMAND_ROWS);
                        let new_scroll_offset = (*scroll_offset + 1).min(max_scroll);
                        app.panels.cheat_console = CheatConsoleState::Open {
                            input: input.clone(),
                            cursor_position: *cursor_position,
                            autocomplete_suggestions: autocomplete_suggestions.clone(),
                            autocomplete_index: *autocomplete_index,
                            scroll_offset: new_scroll_offset,
                        };
                    }
                    return Ok(true);
                }
                _ => return Ok(true), // Consume all other input while console is open
            }
        }
    }

    Ok(false) // Didn't handle input
}

/// Run a command typed into the in-game console. Failures are shown in the message log;
/// they must never propagate, or they would end the interactive client's main loop.
fn execute_command(app: &mut App, command: &str) {
    if let Err(e) = get_command_registry().execute(app, command) {
        app.core.game.res.log_red(format!("Cheat failed: {e}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const P: Position = Position {
        x: 10,
        y: 10,
        z: -19,
    };

    #[test]
    fn positions_can_be_absolute_relative_or_mixed() {
        assert_eq!(
            parse_position(&["1", "-2", "3"], P),
            Ok(Position { x: 1, y: -2, z: 3 })
        );
        assert_eq!(parse_position(&["~", "~", "~"], P), Ok(P));
        assert_eq!(
            parse_position(&["~-2", "~1", "~"], P),
            Ok(Position {
                x: 8,
                y: 11,
                z: -19
            })
        );
        assert_eq!(
            parse_position(&["~5", "~", "0"], P),
            Ok(Position { x: 15, y: 10, z: 0 })
        );
    }

    #[test]
    fn teleporting_loads_the_new_place_before_the_first_step() {
        use lithicrivers_core::systems::fog_of_war::is_tile_visited;
        let mut runner = crate::headless::chain_commands(&["skip-splash", "key ui.MENU_PREV"]);
        let tick = runner.app.core.game.res.time.tick;
        let (x, y, z): (i64, i64, i64) = (700, 700, -5);
        let chunk = (
            x.div_euclid(lithicrivers_core::world::CHUNK_SIZE),
            y.div_euclid(lithicrivers_core::world::CHUNK_SIZE),
            z,
        );
        assert!(!runner
            .app
            .core
            .game
            .res
            .is_chunk_explored(chunk.0, chunk.1, chunk.2));
        runner.chain(&[&format!("cheat tp {x} {y} {z}")]);

        let game = &runner.app.core.game;
        assert_eq!(game.res.time.tick, tick, "teleporting passes no time");
        assert!(game.res.is_chunk_explored(chunk.0, chunk.1, chunk.2));
        assert!(is_tile_visited(&game.world, x, y, z));
        assert_eq!(
            (
                runner.app.ui.view_x,
                runner.app.ui.view_y,
                runner.app.ui.view_z
            ),
            (x, y, z)
        );
    }

    #[test]
    fn bad_positions_are_errors() {
        assert!(parse_position(&["~x", "0", "0"], P).is_err());
        assert!(parse_position(&["1", "2"], P).is_err());
        assert!(parse_position(&["1", "2", "3", "4"], P).is_err());
        assert!(parse_position(&["1", "two", "3"], P).is_err());
    }

    #[test]
    fn switches_suggest_on_and_off() {
        assert_eq!(
            update_autocomplete("keepbody ").0,
            vec!["keepbody on", "keepbody off"]
        );
        assert_eq!(
            update_autocomplete("keepinventory of").0,
            vec!["keepinventory off"]
        );
        assert!(update_autocomplete("keepbody on ").0.is_empty());
    }

    #[test]
    fn test_find_common_prefix() {
        // Test empty list
        assert_eq!(find_common_prefix(&[]), "");

        // Test single item
        assert_eq!(find_common_prefix(&["test".to_string()]), "test");

        // Test multiple items with common prefix
        let items = vec!["toggle_fogofwar".to_string(), "toggle_tutorial".to_string()];
        assert_eq!(find_common_prefix(&items), "toggle_");

        // Test multiple items with no common prefix
        let items = vec!["tp".to_string(), "noclip_toggle".to_string()];
        assert_eq!(find_common_prefix(&items), "");

        // Test items where one is a prefix of another
        let items = vec!["test".to_string(), "testing".to_string()];
        assert_eq!(find_common_prefix(&items), "test");
    }
}

#[cfg(test)]
mod suggestion_tests {
    use crate::app_state::CheatConsoleState;
    use crate::headless::chain_commands;

    #[test]
    fn every_giveitem_suggestion_can_be_reached_and_used() {
        let names = lithicrivers_core::components::get_all_item_sprite_names();
        let last = names.last().unwrap();
        let mut runner = chain_commands(&[
            "skip-splash",
            "key ui.OPEN_COMMAND_MENU",
            "type \"giveitem \"",
            &format!("key ui.CHEAT_CONSOLE_SCROLL_DOWN {}", names.len() + 3),
        ]);
        assert!(
            runner.screen_text().contains(&format!("> giveitem {last}")),
            "the last item is picked and visible"
        );
        // Enter fills in the pick; the next Enter runs it
        runner.chain(&["rawkey ENTER"]);
        let CheatConsoleState::Open { input, .. } = &runner.app.panels.cheat_console else {
            panic!("still open after picking");
        };
        assert_eq!(input, &format!("giveitem {last}"));
    }
}

#[cfg(test)]
mod set_time_tests {
    use crate::headless::{chain_commands, parse_script};

    #[test]
    fn set_time_skips_forward_to_the_next_matching_time() {
        let mut runner = chain_commands(&["skip-splash", "cheat set_time 00:00"]);
        let time = &runner.app.core.game.res.time;
        assert_eq!(time.clock(), (0, 0));
        assert!(time.is_night());
        let midnight = time.tick;
        assert!(midnight > 0, "forward from 08:00, never back");
        runner.chain(&[
            "expect-text \"The time is now 00:00\"",
            "cheat set_time 12:30",
        ]);
        let time = &runner.app.core.game.res.time;
        assert_eq!(time.clock(), (12, 30));
        assert!(time.tick > midnight);
    }

    #[test]
    fn set_time_refuses_a_bad_time() {
        let mut runner = chain_commands(&["skip-splash"]);
        let before = runner.app.core.game.res.time.tick;
        for bad in ["25:99", "noon", "12"] {
            let script = parse_script(&format!("cheat set_time {bad}")).unwrap();
            let report = runner.run_steps(&script.steps);
            let error = report.error.unwrap_or_default();
            assert!(error.contains("Usage: set_time hh:mm"), "{bad}: {error}");
            assert_eq!(runner.app.core.game.res.time.tick, before);
        }
    }
}

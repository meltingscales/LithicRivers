//! Mouse input: a crossterm mouse event (or a headless `click` / `scroll` / `move` step)
//! becomes a `MouseInput`, and the actions it is bound to (the `mouse` keybind category)
//! call the same code as the keys. The keyboard stays complete on its own.

use crate::app::input::world_build::{break_at, has_item_in_inventory, place_from_hotbar};
use crate::{App, MenuTab, SplashState};
use crossterm::event::KeyCode;
use lithicrivers_core::components::{itemkind_name, ItemKind, Position};
use lithicrivers_core::factory::FactoryBlock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseKind {
    Click(MouseButton),
    ScrollUp,
    ScrollDown,
    Move,
}

/// One mouse event at a screen cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseInput {
    pub kind: MouseKind,
    pub col: u16,
    pub row: u16,
    pub shift: bool,
}

impl MouseInput {
    /// The binding name this event matches in the `mouse` keybind category.
    pub fn binding_name(&self) -> Option<String> {
        let base = match self.kind {
            MouseKind::Click(MouseButton::Left) => "LEFT_CLICK",
            MouseKind::Click(MouseButton::Right) => "RIGHT_CLICK",
            MouseKind::Click(MouseButton::Middle) => "MIDDLE_CLICK",
            MouseKind::ScrollUp => "SCROLL_UP",
            MouseKind::ScrollDown => "SCROLL_DOWN",
            MouseKind::Move => return None,
        };
        Some(if self.shift {
            format!("SHIFT+{base}")
        } else {
            base.to_string()
        })
    }
}

/// Whether `input` is bound to the mouse action `action` in the config.
fn bound(app: &App, action: &str, input: &MouseInput) -> bool {
    let Some(name) = input.binding_name() else {
        return false;
    };
    app.core
        .config_manager
        .get_keybind("mouse", action)
        .and_then(|v| v.as_array())
        .is_some_and(|list| list.iter().any(|b| b.as_str() == Some(name.as_str())))
}

/// Whether a window or menu over the world has the input.
fn modal_open(app: &App) -> bool {
    use crate::app_state::*;
    app.panels.furnace_window.is_some()
        || app.panels.filter_picker.is_some()
        || app.panels.blueprint_window.is_some()
        || app.panels.stairs_prompt.is_some()
        || crate::app::input::death::showing(app)
        || app.panels.power_info.is_some()
        || app.panels.cheat_console.is_open()
        || !matches!(app.panels.corpse_looting, CorpseLootingState::None)
        || !matches!(app.panels.hotbar_assignment, HotbarAssignmentState::None)
        || !matches!(app.panels.npc_interaction, NPCInteractionState::None)
        || !matches!(app.panels.multi_action_select, MultiActionSelectState::None)
        || app.combat.is_active()
}

/// A mouse event from the player. The demo ignores it, as it does keys.
pub fn handle_mouse(app: &mut App, input: MouseInput) {
    if app.demo.is_none() {
        dispatch_mouse(app, input);
    }
}

/// Route a mouse event. The demo plays its scripted mouse steps through here.
pub fn dispatch_mouse(app: &mut App, input: MouseInput) {
    app.ui.mouse_cell = Some((input.col, input.row));
    if input.kind == MouseKind::Move {
        // In Look mode, hovering picks what to look at; only the keys move the view, so
        // the view never drifts under a still mouse
        if app.panels.look.mode && app.ui.current_tab == MenuTab::World && !modal_open(app) {
            if let Some(tile) = crate::ui::panels::world::screen_to_tile(app, input.col, input.row)
            {
                app.panels.look.cursor = tile;
            }
        }
        // In Blueprint mode the cursor follows the mouse over the world
        if blueprint_on_world(app) {
            if let Some(tile) = crate::ui::panels::world::screen_to_tile(app, input.col, input.row)
            {
                crate::app::input::blueprint::set_cursor(app, tile);
            }
        }
        return;
    }

    // Menus, lists and windows first: whatever recorded its rows while drawing
    if let MouseKind::Click(button) = input.kind {
        use crate::ui::click_targets::Resolved;
        match app.ui.click_targets.resolve(input.col, input.row) {
            Resolved::Target(target) => {
                click_target(app, target, button);
                return;
            }
            Resolved::Outside => {
                close_window(app);
                return;
            }
            Resolved::Nothing => {}
        }
    }
    if app.splash.state != SplashState::MainUI {
        return;
    }

    // Scrolling over the menu bar does nothing
    let on_menu = app
        .ui
        .bottom_menu_rect
        .is_some_and(|r| (r.y..r.y + r.height).contains(&input.row));
    if on_menu {
        return;
    }

    let on_world = app.ui.current_tab == MenuTab::World && !modal_open(app);
    if matches!(input.kind, MouseKind::ScrollUp | MouseKind::ScrollDown) {
        if on_world {
            scroll_world(app, &input);
        } else {
            // Lists and menus scroll the way their up/down keys move them
            let down = input.kind == MouseKind::ScrollDown;
            let action = if app.panels.cheat_console.is_open() {
                if down {
                    "CHEAT_CONSOLE_SCROLL_DOWN"
                } else {
                    "CHEAT_CONSOLE_SCROLL_UP"
                }
            } else if down {
                "CREDITS_SCROLL_DOWN"
            } else {
                "CREDITS_SCROLL_UP"
            };
            if let Some(code) = key_for(app, "ui", action) {
                let _ = crate::app::input_handler::dispatch(app, code);
            }
        }
        return;
    }
    if !on_world {
        return;
    }
    let Some(tile) = crate::ui::panels::world::screen_to_tile(app, input.col, input.row) else {
        return;
    };
    if blueprint_on_world(app) && bound(app, "PLACE", &input) {
        crate::app::input::blueprint::set_cursor(app, tile);
        crate::app::input::blueprint::act_at(app, tile);
    } else if bound(app, "PICK_BLOCK", &input) {
        pick_block(app, tile);
    } else if bound(app, "PLACE", &input) {
        if !in_reach(app, tile) {
            look_at(app, tile);
            app.core.game.res.log("Too far to build there");
        } else if !interact_at(app, tile) {
            let selected =
                app.panels.build.hotbar_assignments[app.panels.build.selected_hotbar_slot];
            if selected.is_some() {
                place_from_hotbar(app, tile.x, tile.y, tile.z);
            } else {
                look_at(app, tile);
            }
        }
    } else if bound(app, "BREAK", &input) {
        if in_reach(app, tile) {
            break_at(app, tile.x, tile.y, tile.z);
        } else {
            look_at(app, tile);
            app.core.game.res.log("Too far to mine there");
        }
    }
}

fn blueprint_on_world(app: &App) -> bool {
    app.panels.build.mode == crate::app_state::BuildMode::Blueprint
        && app.ui.current_tab == MenuTab::World
        && !modal_open(app)
}

/// Press the first key bound to `category.action`, as if the player had.
fn press(app: &mut App, category: &str, action: &str) {
    if let Some(code) = key_for(app, category, action) {
        let _ = crate::app::input_handler::dispatch(app, code);
    }
}

/// A click outside the topmost window closes it, as the back or close key does.
fn close_window(app: &mut App) {
    if app.splash.state == SplashState::MainUI {
        press(app, "ui", "CLOSE_HELP_MENU");
    } else {
        press(app, "title", "BACK");
    }
}

/// Click a recorded target: select its row, then do what the menu's Enter key does there.
/// A right click on an item row moves the whole stack.
fn click_target(app: &mut App, target: crate::ui::click_targets::ClickTarget, button: MouseButton) {
    use crate::app::title_menu::CreateField;
    use crate::app_state::*;
    use crate::ui::click_targets::ClickTarget as T;
    let select_title = |app: &mut App| press(app, "title", "SELECT");
    let activate = |app: &mut App| press(app, "ui", "MENU_ACTIVATE");
    match target {
        T::ModalArea => {}
        T::Tab(tab) => {
            app.ui.current_tab = tab;
            app.activate_menu();
        }
        T::TitleChoice(i) => {
            app.title.selected = i;
            select_title(app);
        }
        T::WorldRow(i) => {
            app.title.world_selected = i;
            select_title(app);
        }
        T::WorldAction(i) => {
            app.title.world_action = i;
            select_title(app);
        }
        T::DeleteChoice(i) => {
            app.title.delete_choice = i;
            select_title(app);
        }
        T::CreateField(i) => {
            app.title.create.field = i;
            // Text fields only take focus; toggles flip, World type cycles, buttons press
            let field = app.title.create.fields().get(i).copied();
            if !matches!(field, Some(CreateField::Name | CreateField::Seed) | None) {
                select_title(app);
            }
        }
        T::OptionsRow(i) => {
            app.title.options_selected = i;
            if i + 1 == crate::app::title_menu::OPTIONS_ROWS.len() {
                select_title(app);
            }
        }
        T::InteractionChoice(i) => {
            if let MultiActionSelectState::SelectingAction {
                selected_action, ..
            } = &mut app.panels.multi_action_select
            {
                *selected_action = i;
            }
            activate(app);
        }
        T::DialogueChoice(i) => {
            if let NPCInteractionState::InDialogue {
                selected_choice, ..
            } = &mut app.panels.npc_interaction
            {
                *selected_choice = i;
            }
            activate(app);
        }
        T::FilterRow(i) => {
            if let Some(picker) = &mut app.panels.filter_picker {
                picker.selected = i;
            }
            activate(app);
        }
        T::BlockPickerRow(i) => {
            if let HotbarAssignmentState::ChoosingBlock { selected_block, .. } =
                &mut app.panels.hotbar_assignment
            {
                *selected_block = i;
            }
            activate(app);
        }
        T::CorpsePickerRow(i) => {
            if let CorpseLootingState::SelectingCorpse {
                selected_corpse, ..
            } = &mut app.panels.corpse_looting
            {
                *selected_corpse = i;
            }
            activate(app);
        }
        T::CombatMove(i) => {
            if let CombatUiState::Active { current_move, .. } = &mut app.combat {
                *current_move = i;
            }
            activate(app);
        }
        T::Suggestion(i) => {
            if let CheatConsoleState::Open {
                autocomplete_index, ..
            } = &mut app.panels.cheat_console
            {
                *autocomplete_index = Some(i);
            }
            // Enter on a pick fills it in
            let _ = crate::app::input_handler::dispatch(app, KeyCode::Enter);
        }
        T::TransferRow { container, row } => {
            if let CorpseLootingState::LootingCorpse {
                selected_loot_item,
                selected_player_item,
                loot_panel_focus,
                ..
            } = &mut app.panels.corpse_looting
            {
                *loot_panel_focus = container;
                if container {
                    *selected_loot_item = row;
                } else {
                    *selected_player_item = row;
                }
            }
            if button == MouseButton::Right {
                move_whole_stack(app);
            } else {
                activate(app);
            }
        }
        T::FurnaceSlot(i) => {
            if let Some(window) = &mut app.panels.furnace_window {
                if let Some(&slot) = lithicrivers_core::furnace::Slot::for_kind(window.kind).get(i) {
                    window.slot = slot;
                }
                window.player_focus = false;
            }
            if button == MouseButton::Right {
                move_whole_stack(app);
            } else {
                activate(app);
            }
        }
        T::FurnacePlayerRow(i) => {
            if let Some(window) = &mut app.panels.furnace_window {
                window.player_item = i;
                window.player_focus = true;
            }
            if button == MouseButton::Right {
                move_whole_stack(app);
            } else {
                activate(app);
            }
        }
        T::RecipeRow(i) => {
            // The first click selects; clicking the selected recipe crafts it
            if app.panels.crafting.selected == i {
                activate(app);
            } else {
                app.panels.crafting.selected = i;
            }
        }
        T::DictionaryRow(i) => app.panels.item_dictionary.selected = i,
        T::DeathChoice(i) => {
            app.panels.death_choice = i;
            activate(app);
        }
        T::StairsChoice(i) => {
            if let Some(prompt) = &mut app.panels.stairs_prompt {
                prompt.selected = i;
            }
            activate(app);
        }
        T::BlueprintRow(i) => {
            app.core.game.res.blueprints.selected = i;
            activate(app);
        }
    }
}

/// Right click on an item row: press Activate until the clicked stack is used up, a move
/// is refused, or the selection lands on a different item, so the window's own rules (and
/// messages) apply to every item moved.
fn move_whole_stack(app: &mut App) {
    let Some(kind) = selected_kind(app) else {
        return;
    };
    for _ in 0..10_000 {
        let before = format!("{:?}", inventory(app));
        press(app, "ui", "MENU_ACTIVATE");
        if format!("{:?}", inventory(app)) == before || selected_kind(app) != Some(kind) {
            break;
        }
    }
}

fn inventory(app: &App) -> Vec<lithicrivers_core::components::ItemStack> {
    app.core
        .game
        .get_player_entity()
        .and_then(|p| {
            app.core
                .game
                .world
                .get::<&lithicrivers_core::components::Inventory>(p)
                .ok()
                .map(|i| i.slots.clone())
        })
        .unwrap_or_default()
}

/// The item kind on the selected row of the open item window.
fn selected_kind(app: &App) -> Option<ItemKind> {
    use crate::app::item_filter::PlayerRows;
    use crate::app_state::CorpseLootingState;
    use lithicrivers_core::components::Inventory;
    let world = &app.core.game.world;
    let player = app.core.game.get_player_entity()?;
    let stack_kind = |entity, slot: usize| {
        world
            .get::<&Inventory>(entity)
            .ok()
            .and_then(|i| i.slots.get(slot).map(|s| s.kind))
    };
    if let Some(window) = app.panels.furnace_window {
        return if window.player_focus {
            let rows = PlayerRows::for_furnace_slot(world, player, window.slot);
            rows.slot(window.player_item)
                .and_then(|slot| stack_kind(player, slot))
        } else {
            world
                .get::<&lithicrivers_core::furnace::Furnace>(window.entity)
                .ok()
                .and_then(|f| f.slot(window.slot))
                .map(|s| s.kind)
        };
    }
    if let CorpseLootingState::LootingCorpse {
        entity,
        selected_loot_item,
        selected_player_item,
        loot_panel_focus,
    } = &app.panels.corpse_looting
    {
        return if *loot_panel_focus {
            stack_kind(*entity, *selected_loot_item)
        } else {
            let rows = PlayerRows::for_container(world, player, *entity);
            rows.slot(*selected_player_item)
                .and_then(|slot| stack_kind(player, slot))
        };
    }
    None
}

fn key_for(app: &App, category: &str, action: &str) -> Option<KeyCode> {
    app.ui
        .keybinds
        .get_map()
        .get(&format!("{category}:{action}"))
        .and_then(|codes| codes.first().copied())
}

/// Scroll over the world: rotate the next placement, or cycle the hotbar slot.
fn scroll_world(app: &mut App, input: &MouseInput) {
    if bound(app, "ROTATE", input) {
        crate::app::input::world_build::rotate_placement(app);
        return;
    }
    let slots = app.panels.build.hotbar_assignments.len();
    let slot = app.panels.build.selected_hotbar_slot;
    let slot = if bound(app, "NEXT_SLOT", input) {
        (slot + 1) % slots
    } else if bound(app, "PREV_SLOT", input) {
        (slot + slots - 1) % slots
    } else {
        return;
    };
    app.panels.build.selected_hotbar_slot = slot;
    app.core
        .game
        .res
        .log(format!("Selected hotbar slot: {}", slot + 1));
}

/// How far the mouse builds and breaks, in tiles (Chebyshev, so an 11x11 square around
/// the player): farther than the keyboard's 8 neighbours, so building with the mouse is quick.
pub const MOUSE_REACH: i64 = 5;

/// Within `MOUSE_REACH` of the player, on their level, and not their own tile.
fn in_reach(app: &App, tile: Position) -> bool {
    app.core.game.get_player_position().is_some_and(|p| {
        p.z == tile.z
            && (p.x - tile.x).abs() <= MOUSE_REACH
            && (p.y - tile.y).abs() <= MOUSE_REACH
            && p != tile
    })
}

/// Point look mode's cursor at the tile, so the look panel shows it.
fn look_at(app: &mut App, tile: Position) {
    app.panels.look.mode = true;
    app.panels.look.cursor = tile;
}

/// Run the first interaction at the tile (open a chest, talk, fuel a generator...).
/// Returns whether there was one.
fn interact_at(app: &mut App, tile: Position) -> bool {
    let action = crate::app::input::world_action::nearby_interactions(app)
        .and_then(|n| n.actions.into_iter().find(|a| a.position() == tile));
    match action {
        Some(action) => {
            crate::app::input::world_action::execute_interaction_action(app, &action);
            true
        }
        None => false,
    }
}

/// The item a block was placed from: a factory block's item, or a placed tile's item.
fn item_at(
    app: &mut App,
    tile: Position,
) -> Option<(ItemKind, Option<lithicrivers_core::factory::Facing>)> {
    if let Some(&entity) = app.core.game.res.factory_index.get(&tile) {
        let block = *app.core.game.world.get::<&FactoryBlock>(entity).ok()?;
        let facing = block.kind.directional().then_some(block.facing);
        return block.kind.item().map(|item| (item, facing));
    }
    let tile_kind = app
        .core
        .game
        .res
        .world_state
        .world
        .get_tile_cached(tile.x, tile.y, tile.z);
    lithicrivers_core::crafting::placeable_items()
        .into_iter()
        .find(|&item| lithicrivers_core::tiles::TileKind::from_item_kind(item) == Some(tile_kind))
        .map(|item| (item, None))
}

/// Middle click: select the hotbar slot holding the clicked block's item (or put the item
/// in the current slot if the player has some), and copy the block's facing.
fn pick_block(app: &mut App, tile: Position) {
    let Some((item, facing)) = item_at(app, tile) else {
        return;
    };
    let build = &mut app.panels.build;
    if let Some(slot) = build
        .hotbar_assignments
        .iter()
        .position(|s| *s == Some(item))
    {
        build.selected_hotbar_slot = slot;
    } else if has_item_in_inventory(app, item) {
        let build = &mut app.panels.build;
        build.hotbar_assignments[build.selected_hotbar_slot] = Some(item);
    } else {
        app.core
            .game
            .res
            .log(format!("You have no {}", itemkind_name(item)));
        return;
    }
    if let Some(facing) = facing {
        app.panels.build.placement_facing = facing;
    }
    app.core
        .game
        .res
        .log(format!("Picked {}", itemkind_name(item)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headless::{chain_commands, Runner};
    use crate::ui::click_targets::ClickTarget;
    use crate::ui::panels::world::cell_to_tile;

    /// The screen cell of the tile `dx`, `dy` from the player, at the current zoom.
    fn cell(runner: &mut Runner, dx: i64, dy: i64) -> (u16, u16) {
        let _ = runner.screen_text();
        let area = runner.app.ui.world_view_rect.unwrap();
        let scale = runner.app.ui.scale.as_i64();
        let col = area.x as i64 + area.width as i64 / 2 + dx * scale;
        let row = area.y as i64 + area.height as i64 / 2 + dy * scale;
        (col as u16, row as u16)
    }

    fn click(runner: &mut Runner, dx: i64, dy: i64, button: &str) {
        let (c, r) = cell(runner, dx, dy);
        runner.chain(&[&format!("click {c} {r} {button}")]);
    }

    fn last_message(runner: &Runner) -> String {
        let log = &runner.app.core.game.res.events.get_message_log().messages;
        log.last().map(|m| m.text.clone()).unwrap_or_default()
    }

    /// On open grass, with a chest on hotbar slot 1 in Place mode.
    const WITH_CHEST: &[&str] = &[
        "skip-splash",
        "key ui.MENU_PREV",
        "cheat tp -15 0 0",
        "cheat giveitem chest 2",
        "key build.TOGGLE_BREAK_PLACE_MODE 2",
        // Picker: Clear slot, Acorn, Chest
        "key hotbar.HOTBAR_SLOT_1 2",
        "key movement.MOVE_SOUTH 2",
        "key ui.MENU_ACTIVATE",
    ];

    fn block(runner: &Runner, x: i64, y: i64) -> Option<lithicrivers_core::factory::FactoryKind> {
        let game = &runner.app.core.game;
        game.res
            .factory_index
            .get(&Position { x, y, z: 0 })
            .and_then(|&e| game.world.get::<&FactoryBlock>(e).ok().map(|b| b.kind))
    }

    #[test]
    fn cells_map_to_tiles_at_every_zoom() {
        // A 10x6 view centered on (0, 0)
        assert_eq!(cell_to_tile((0, 0), (10, 6), 1, (5, 3)), (0, 0, 0, 0));
        assert_eq!(cell_to_tile((0, 0), (10, 6), 1, (0, 0)), (-5, -3, 0, 0));
        assert_eq!(cell_to_tile((0, 0), (10, 6), 2, (7, 3)), (1, 0, 0, 0));
        assert_eq!(cell_to_tile((0, 0), (10, 6), 2, (8, 4)), (1, 0, 1, 1));
        assert_eq!(cell_to_tile((4, -2), (10, 6), 3, (2, 3)), (3, -2, 0, 0));
        let mut runner = chain_commands(&["skip-splash", "key ui.MENU_PREV"]);
        let _ = runner.screen_text();
        let area = runner.app.ui.world_view_rect.unwrap();
        assert_eq!(
            crate::ui::panels::world::screen_to_tile(&runner.app, area.x + area.width, area.y),
            None,
            "outside the view"
        );
    }

    #[test]
    fn left_click_places_the_hotbar_block() {
        let mut runner = chain_commands(WITH_CHEST);
        click(&mut runner, 1, 0, "left");
        assert_eq!(
            block(&runner, -14, 0),
            Some(lithicrivers_core::factory::FactoryKind::Chest)
        );
        // Clicking the chest now opens it rather than placing another
        click(&mut runner, 1, 0, "left");
        assert!(!matches!(
            runner.app.panels.corpse_looting,
            crate::app_state::CorpseLootingState::None
        ));
    }

    #[test]
    fn builds_up_to_five_tiles_away_and_looks_beyond() {
        let mut runner = chain_commands(WITH_CHEST);
        let reach = MOUSE_REACH;
        click(&mut runner, reach, -reach, "left");
        assert_eq!(
            block(&runner, -15 + reach, -reach),
            Some(lithicrivers_core::factory::FactoryKind::Chest),
            "the far corner of the reach"
        );
        click(&mut runner, reach, -reach, "right");
        assert_eq!(
            block(&runner, -15 + reach, -reach),
            None,
            "and breaks there too"
        );
        click(&mut runner, reach + 1, 0, "left");
        assert_eq!(block(&runner, -15 + reach + 1, 0), None);
        assert!(last_message(&runner).ends_with("Too far to build there"));
        assert!(runner.app.panels.look.mode);
        assert_eq!(
            runner.app.panels.look.cursor,
            Position {
                x: -15 + reach + 1,
                y: 0,
                z: 0
            }
        );
        click(&mut runner, 0, reach + 1, "right");
        assert!(last_message(&runner).ends_with("Too far to mine there"));
    }

    #[test]
    fn right_click_breaks_within_reach() {
        let mut runner = chain_commands(WITH_CHEST);
        click(&mut runner, -1, 0, "left");
        assert!(block(&runner, -16, 0).is_some());
        click(&mut runner, -1, 0, "right");
        assert_eq!(block(&runner, -16, 0), None, "broken");
        let chests = runner.app.core.game.get_player_entity().map(|p| {
            runner
                .app
                .core
                .game
                .world
                .get::<&lithicrivers_core::components::Inventory>(p)
                .unwrap()
                .slots
                .iter()
                .filter(|s| s.kind == ItemKind::Chest)
                .map(|s| s.qty)
                .sum::<u32>()
        });
        assert_eq!(chests, Some(2), "the chest came back");
    }

    #[test]
    fn scroll_cycles_the_hotbar_and_shift_scroll_rotates() {
        let mut runner = chain_commands(WITH_CHEST);
        let (c, r) = cell(&mut runner, 0, 0);
        runner.chain(&[&format!("scroll down {c} {r}")]);
        assert_eq!(runner.app.panels.build.selected_hotbar_slot, 1);
        runner.chain(&[&format!("scroll up {c} {r}"), &format!("scroll up {c} {r}")]);
        assert_eq!(runner.app.panels.build.selected_hotbar_slot, 11, "wraps");
        runner.chain(&[&format!("scroll down {c} {r} shift")]);
        assert_eq!(
            runner.app.panels.build.placement_facing,
            lithicrivers_core::factory::Facing::East
        );
        assert_eq!(
            runner.app.panels.build.selected_hotbar_slot, 11,
            "rotating keeps the slot"
        );
    }

    #[test]
    fn middle_click_picks_a_block_and_its_facing() {
        let mut runner = chain_commands(&[
            "skip-splash",
            "key ui.MENU_PREV",
            "cheat tp -15 0 0",
            "cheat giveitem wooden_conveyor 2",
        ]);
        let p = runner.app.core.game.get_player_entity().unwrap();
        let _ = p;
        {
            let g = &mut runner.app.core.game;
            lithicrivers_core::factory::place(
                &mut g.world,
                &mut g.res,
                lithicrivers_core::factory::FactoryKind::Belt,
                Position {
                    x: -15,
                    y: -2,
                    z: 0,
                },
                lithicrivers_core::factory::Facing::West,
            )
            .unwrap();
            lithicrivers_core::factory::place(
                &mut g.world,
                &mut g.res,
                lithicrivers_core::factory::FactoryKind::Chest,
                Position {
                    x: -13,
                    y: -2,
                    z: 0,
                },
                lithicrivers_core::factory::Facing::North,
            )
            .unwrap();
        }
        click(&mut runner, 0, -2, "middle");
        let build = &runner.app.panels.build;
        assert_eq!(
            build.hotbar_assignments[build.selected_hotbar_slot],
            Some(ItemKind::WoodenConveyor)
        );
        assert_eq!(
            build.placement_facing,
            lithicrivers_core::factory::Facing::West
        );
        click(&mut runner, 2, -2, "middle");
        assert!(last_message(&runner).ends_with("You have no Chest"));
    }

    #[test]
    fn clicking_a_tab_opens_it_and_scrolling_moves_lists() {
        let mut runner = chain_commands(&["skip-splash", "key ui.MENU_PREV"]);
        let _ = runner.screen_text();
        let menu = runner.app.ui.bottom_menu_rect.unwrap();
        let col = (menu.x..menu.x + menu.width)
            .find(|&c| {
                matches!(
                    runner.app.ui.click_targets.hit(c, menu.y + 1),
                    Some((_, ClickTarget::Tab(MenuTab::ItemDictionary)))
                )
            })
            .expect("the Items tab is on the bar");
        runner.chain(&[&format!("click {col} {} left", menu.y + 1)]);
        assert_eq!(runner.app.ui.current_tab, MenuTab::ItemDictionary);
        runner.chain(&["scroll down 30 10", "scroll down 30 10"]);
        assert_eq!(runner.app.panels.item_dictionary.selected, 2);
    }

    #[test]
    fn mouse_bindings_come_from_the_config() {
        let mut runner = chain_commands(WITH_CHEST);
        runner.app.core.config_manager.data.keybinds["mouse"]["PLACE"] = serde_json::json!([]);
        click(&mut runner, 1, 0, "left");
        assert_eq!(
            block(&runner, -14, 0),
            None,
            "an unbound button does nothing"
        );
    }

    /// The screen cell where `target` was drawn on the last frame.
    fn target_cell(runner: &mut Runner, target: ClickTarget) -> (u16, u16) {
        let screen = runner.screen_text();
        let rows = screen.lines().count() as u16;
        let cols = screen.lines().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
        for row in 0..rows {
            for col in 0..cols {
                if runner.app.ui.click_targets.hit(col, row).map(|(_, t)| t) == Some(target) {
                    return (col, row);
                }
            }
        }
        panic!("{target:?} is not on screen:\n{}", runner.screen_text());
    }

    fn click_on(runner: &mut Runner, target: ClickTarget, button: &str) {
        let (c, r) = target_cell(runner, target);
        runner.chain(&[&format!("click {c} {r} {button}")]);
    }

    const TITLE: &[&str] = &["rawkey ENTER", "rawkey ENTER"];

    #[test]
    fn clicking_watch_demo_starts_the_demo() {
        let mut runner = chain_commands(TITLE);
        assert_eq!(runner.app.splash.state, SplashState::Title);
        click_on(&mut runner, ClickTarget::TitleChoice(1), "left");
        assert!(runner.app.demo.is_some());
    }

    #[test]
    fn clicking_through_the_create_world_form() {
        let mut runner = chain_commands(TITLE);
        click_on(&mut runner, ClickTarget::TitleChoice(0), "left");
        assert_eq!(runner.app.splash.state, SplashState::WorldList);
        // With no worlds, the first row creates one
        click_on(&mut runner, ClickTarget::WorldRow(0), "left");
        assert_eq!(runner.app.splash.state, SplashState::CreateWorld);
        click_on(&mut runner, ClickTarget::CreateField(2), "left");
        assert!(
            runner.screen_text().contains("Allow cheats: ON"),
            "{}",
            runner.screen_text()
        );
        // Clicking the name field only focuses it
        click_on(&mut runner, ClickTarget::CreateField(0), "left");
        assert_eq!(runner.app.splash.state, SplashState::CreateWorld);
        assert_eq!(runner.app.title.create.field, 0);
    }

    #[test]
    fn clicking_a_block_picker_row_assigns_it() {
        let mut runner = chain_commands(&WITH_CHEST[..6]);
        // Picker: Clear slot, Acorn, Chest
        click_on(&mut runner, ClickTarget::BlockPickerRow(2), "left");
        let build = &runner.app.panels.build;
        assert_eq!(
            build.hotbar_assignments[build.selected_hotbar_slot],
            Some(ItemKind::Chest)
        );
    }

    /// A placed chest, open, with the player's items on the right.
    fn open_chest() -> Runner {
        let mut runner = chain_commands(WITH_CHEST);
        click(&mut runner, 1, 0, "left");
        click(&mut runner, 1, 0, "left");
        assert!(matches!(
            runner.app.panels.corpse_looting,
            CorpseLootingState::LootingCorpse { .. }
        ));
        runner
    }

    /// The player's stacks, in inventory order.
    fn player_stacks(runner: &Runner) -> Vec<(ItemKind, u32)> {
        let game = &runner.app.core.game;
        let player = game.get_player_entity().unwrap();
        game.world
            .get::<&lithicrivers_core::components::Inventory>(player)
            .map(|inv| inv.slots.iter().map(|s| (s.kind, s.qty)).collect())
            .unwrap_or_default()
    }

    fn player_count(runner: &Runner, kind: ItemKind) -> u32 {
        player_stacks(runner)
            .into_iter()
            .filter(|&(k, _)| k == kind)
            .map(|(_, n)| n)
            .sum()
    }

    use crate::app_state::{CheatConsoleState, CombatUiState, CorpseLootingState};

    #[test]
    fn clicking_outside_a_window_closes_it() {
        let mut runner = open_chest();
        runner.chain(&["click 0 0 left"]);
        assert!(matches!(
            runner.app.panels.corpse_looting,
            CorpseLootingState::None
        ));
    }

    #[test]
    fn right_clicking_an_item_moves_the_whole_stack() {
        let mut runner = open_chest();
        let before = player_count(&runner, ItemKind::Log);
        assert!(before > 1, "the starting kit has logs");
        // A chest takes everything, so its rows are the inventory's stacks
        let row = player_stacks(&runner)
            .iter()
            .position(|&(k, _)| k == ItemKind::Log)
            .unwrap();
        click_on(
            &mut runner,
            ClickTarget::TransferRow {
                container: false,
                row,
            },
            "right",
        );
        assert_eq!(player_count(&runner, ItemKind::Log), 0);
    }

    #[test]
    fn a_full_furnace_slot_refuses_the_rest_of_a_right_clicked_stack() {
        use crate::app::item_filter::PlayerRows;
        use lithicrivers_core::furnace::{Furnace, Slot, FURNACE_SLOT_MAX};
        let mut runner = chain_commands(&[
            "skip-splash",
            "key ui.MENU_PREV",
            "cheat tp 0 0 0",
            "cheat giveitem crude_furnace",
            "cheat giveitem log 40",
            "key build.TOGGLE_BREAK_PLACE_MODE 2",
            "key hotbar.HOTBAR_SLOT_1 2",
            "key movement.MOVE_SOUTH 2",
            "key ui.MENU_ACTIVATE",
            "key hotbar.HOTBAR_SLOT_1",
            "key build.PLACE_WEST",
            "key build.TOGGLE_BREAK_PLACE_MODE 2",
            "key action.INTERACT",
        ]);
        let window = runner
            .app
            .panels
            .furnace_window
            .expect("the furnace window is open");
        click_on(&mut runner, ClickTarget::FurnaceSlot(0), "left");
        let logs = player_count(&runner, ItemKind::Log);
        let game = &runner.app.core.game;
        let player = game.get_player_entity().unwrap();
        let rows = PlayerRows::for_furnace_slot(&game.world, player, Slot::Fuel);
        let row = rows
            .slots
            .iter()
            .position(|&slot| player_stacks(&runner)[slot].0 == ItemKind::Log)
            .unwrap();
        click_on(&mut runner, ClickTarget::FurnacePlayerRow(row), "right");
        let fuel = runner
            .app
            .core
            .game
            .world
            .get::<&Furnace>(window.entity)
            .unwrap()
            .slot(Slot::Fuel)
            .map(|s| s.qty);
        assert_eq!(fuel, Some(FURNACE_SLOT_MAX));
        assert_eq!(
            player_count(&runner, ItemKind::Log),
            logs - FURNACE_SLOT_MAX
        );
    }

    #[test]
    fn clicking_a_recipe_selects_it_and_a_second_click_crafts() {
        let mut runner = chain_commands(&["skip-splash", "key ui.MENU_NEXT 5"]);
        assert_eq!(runner.app.ui.current_tab, MenuTab::Crafting);
        click_on(&mut runner, ClickTarget::RecipeRow(1), "left");
        assert_eq!(runner.app.panels.crafting.selected, 1);
        let recipe = runner.app.panels.crafting.recipe_handler.get_recipes()[1];
        {
            let game = &mut runner.app.core.game;
            let player = game.get_player_entity().unwrap();
            let mut inv = game
                .world
                .get::<&mut lithicrivers_core::components::Inventory>(player)
                .unwrap();
            for &(kind, qty) in recipe.ingredients {
                inv.slots
                    .push(lithicrivers_core::components::ItemStack { kind, qty });
            }
        }
        let before = player_count(&runner, recipe.result);
        click_on(&mut runner, ClickTarget::RecipeRow(1), "left");
        assert_eq!(
            player_count(&runner, recipe.result),
            before + recipe.quantity
        );
    }

    #[test]
    fn clicking_a_scrolled_dictionary_row_selects_that_item() {
        let mut runner = chain_commands(&["skip-splash"]);
        runner.app.ui.current_tab = MenuTab::ItemDictionary;
        for _ in 0..20 {
            runner.chain(&["scroll down 30 10"]);
        }
        let selected = runner.app.panels.item_dictionary.selected;
        assert_eq!(selected, 20);
        click_on(
            &mut runner,
            ClickTarget::DictionaryRow(selected - 2),
            "left",
        );
        assert_eq!(runner.app.panels.item_dictionary.selected, selected - 2);
        assert!(
            runner.app.ui.click_targets.hit(0, 0).is_none(),
            "only rows on screen are recorded"
        );
    }

    #[test]
    fn clicking_a_cheat_suggestion_fills_it_in() {
        let mut runner = chain_commands(&[
            "skip-splash",
            "key ui.OPEN_COMMAND_MENU",
            "type \"giveitem \"",
        ]);
        click_on(&mut runner, ClickTarget::Suggestion(2), "left");
        let CheatConsoleState::Open { input, .. } = &runner.app.panels.cheat_console else {
            panic!("the console is open");
        };
        assert!(
            input.starts_with("giveitem ") && input.len() > "giveitem ".len(),
            "{input}"
        );
    }

    #[test]
    fn clicking_a_combat_move_uses_it() {
        let mut runner = chain_commands(&[
            "skip-splash",
            "cheat tp 0 0 0",
            "cheat spawn_enemy feral_dog",
            "key movement.WAIT",
        ]);
        let messages = runner
            .app
            .core
            .game
            .res
            .events
            .get_message_log()
            .messages
            .len();
        click_on(&mut runner, ClickTarget::CombatMove(1), "left");
        assert!(matches!(
            runner.app.combat,
            CombatUiState::Active {
                current_move: 1,
                ..
            }
        ));
        assert!(
            runner
                .app
                .core
                .game
                .res
                .events
                .get_message_log()
                .messages
                .len()
                > messages
        );
    }

    #[test]
    fn clicking_an_interaction_choice_picks_it() {
        let mut runner = chain_commands(WITH_CHEST);
        runner.chain(&[
            "key build.PLACE_WEST",
            "key build.PLACE_EAST",
            "key build.TOGGLE_BREAK_PLACE_MODE 2",
            "key action.INTERACT",
            "expect-text \"Choose Interaction\"",
        ]);
        click_on(&mut runner, ClickTarget::InteractionChoice(0), "left");
        assert!(matches!(
            runner.app.panels.corpse_looting,
            CorpseLootingState::LootingCorpse { .. }
        ));
    }

    #[test]
    fn clicking_a_dialogue_choice_picks_it() {
        // Tall enough to show the choices under the android's greeting
        let talk = |steps: &[&str]| {
            let mut runner = Runner::new(12345, (160, 50));
            runner.chain(&[
                "skip-splash",
                "key ui.MENU_PREV",
                "key movement.MOVE_EAST",
                "key action.INTERACT",
                "expect-text \"What happened to you?\"",
            ]);
            runner.chain(steps);
            runner
        };
        let mut clicked = talk(&[]);
        click_on(&mut clicked, ClickTarget::DialogueChoice(1), "left");
        let mut keyed = talk(&["key movement.MOVE_SOUTH", "key ui.MENU_ACTIVATE"]);
        let screen = clicked.screen_text();
        assert!(!screen.contains("2.  What happened to you?"), "{screen}");
        assert_eq!(screen, keyed.screen_text());
    }

    #[test]
    fn clicking_a_filter_row_sets_the_filter() {
        let mut runner = chain_commands(&[
            "skip-splash",
            "key ui.MENU_PREV",
            "cheat tp 0 0 0",
            "cheat giveitem filter_extractor",
            "key build.TOGGLE_BREAK_PLACE_MODE 2",
            "key hotbar.HOTBAR_SLOT_1 2",
            "key movement.MOVE_SOUTH 2",
            "key ui.MENU_ACTIVATE",
            "key hotbar.HOTBAR_SLOT_1",
            "key build.PLACE_WEST",
            "key build.TOGGLE_BREAK_PLACE_MODE 2",
            "key action.INTERACT",
            "expect-text \"Filter extractor: pick a filter\"",
        ]);
        click_on(&mut runner, ClickTarget::FilterRow(3), "left");
        assert!(last_message(&runner).ends_with("Filter set to Acorn"));
    }

    #[test]
    fn clicking_through_world_actions_and_delete() {
        // Quitting saves the world and returns to the title
        let mut runner = chain_commands(&[
            "skip-splash",
            "key movement.MOVE_SOUTH",
            "key ui.MENU_NEXT 10",
            "key ui.MENU_ACTIVATE",
        ]);
        click_on(&mut runner, ClickTarget::TitleChoice(0), "left");
        click_on(&mut runner, ClickTarget::WorldRow(0), "left");
        assert_eq!(runner.app.splash.state, SplashState::WorldActions);
        // Clicking outside the actions box backs out of it
        runner.chain(&["click 0 0 left"]);
        assert_eq!(runner.app.splash.state, SplashState::WorldList);
        click_on(&mut runner, ClickTarget::WorldRow(0), "left");
        click_on(&mut runner, ClickTarget::WorldAction(2), "left");
        assert_eq!(runner.app.splash.state, SplashState::ConfirmDelete);
        click_on(&mut runner, ClickTarget::DeleteChoice(0), "left");
        assert_eq!(runner.app.splash.state, SplashState::WorldList);
        assert_eq!(runner.app.title.worlds.len(), 1, "Cancel keeps the world");
    }

    fn underlined(runner: &mut Runner, (col, row): (u16, u16)) -> bool {
        runner
            .buffer()
            .get(col, row)
            .modifier
            .contains(ratatui::style::Modifier::UNDERLINED)
    }

    #[test]
    fn hovering_a_row_underlines_only_that_row() {
        let mut runner = chain_commands(TITLE);
        let demo = target_cell(&mut runner, ClickTarget::TitleChoice(1));
        let play = target_cell(&mut runner, ClickTarget::TitleChoice(0));
        assert!(!underlined(&mut runner, demo));
        runner.chain(&[&format!("move {} {}", demo.0, demo.1)]);
        assert!(underlined(&mut runner, demo));
        assert!(!underlined(&mut runner, play));
    }

    #[test]
    fn rows_under_an_open_window_do_not_underline() {
        let mut runner = open_chest();
        let row = target_cell(
            &mut runner,
            ClickTarget::TransferRow {
                container: false,
                row: 0,
            },
        );
        runner.chain(&[&format!("move {} {}", row.0, row.1)]);
        assert!(underlined(&mut runner, row));
        // The menu bar's tabs sit under the chest window's backdrop
        let menu = runner.app.ui.bottom_menu_rect.unwrap();
        let tab = (menu.x + 3, menu.y + 1);
        runner.chain(&[&format!("move {} {}", tab.0, tab.1)]);
        assert!(!underlined(&mut runner, tab));
    }

    #[test]
    fn hovering_in_look_mode_moves_the_cursor_but_not_the_view() {
        let mut runner =
            chain_commands(&["skip-splash", "key ui.MENU_PREV", "key action.LOOK_TOGGLE"]);
        let _ = runner.screen_text();
        let view = (runner.app.ui.view_x, runner.app.ui.view_y);
        let start = runner.app.panels.look.cursor;
        let (c, r) = cell(&mut runner, 3, -2);
        runner.chain(&[&format!("move {c} {r}")]);
        let cursor = runner.app.panels.look.cursor;
        assert_eq!((cursor.x - start.x, cursor.y - start.y), (3, -2));
        assert_eq!((runner.app.ui.view_x, runner.app.ui.view_y), view);
        // Turning Look mode off brings the view back to the player
        runner.chain(&["key movement.MOVE_EAST", "key action.LOOK_TOGGLE"]);
        let player = runner.app.core.game.get_player_position().unwrap();
        assert_eq!(
            (runner.app.ui.view_x, runner.app.ui.view_y),
            (player.x, player.y)
        );
    }
}

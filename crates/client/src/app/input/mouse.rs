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
        || app.panels.power_info.is_some()
        || app.panels.cheat_console.is_open()
        || !matches!(app.panels.corpse_looting, CorpseLootingState::None)
        || !matches!(app.panels.hotbar_assignment, HotbarAssignmentState::None)
        || !matches!(app.panels.npc_interaction, NPCInteractionState::None)
        || !matches!(app.panels.multi_action_select, MultiActionSelectState::None)
        || app.combat.is_active()
}

/// The menu tab under column `col` of the menu bar, matching how the tab bar lays out its
/// titles: a border, then " title " for each tab with a divider between them.
pub fn tab_at(app: &App, col: u16) -> Option<MenuTab> {
    let rect = app.ui.bottom_menu_rect?;
    let mut x = rect.x + 1;
    for tab in MenuTab::visible(app.ui.tutorial_visible) {
        let width = tab.title().chars().count() as u16 + 2;
        if (x..x + width).contains(&col) {
            return Some(tab);
        }
        x += width + 1;
    }
    None
}

/// A mouse event from the player. The demo ignores it, as it does keys.
pub fn handle_mouse(app: &mut App, input: MouseInput) {
    if app.demo.is_none() {
        dispatch_mouse(app, input);
    }
}

/// Route a mouse event. The demo plays its scripted mouse steps through here.
pub fn dispatch_mouse(app: &mut App, input: MouseInput) {
    if app.splash.state != SplashState::MainUI {
        return;
    }
    app.ui.mouse_cell = Some((input.col, input.row));
    if input.kind == MouseKind::Move {
        return;
    }

    // The menu bar
    let on_menu = app
        .ui
        .bottom_menu_rect
        .is_some_and(|r| (r.y..r.y + r.height).contains(&input.row));
    if on_menu {
        if input.kind == MouseKind::Click(MouseButton::Left) {
            if let Some(tab) = tab_at(app, input.col) {
                app.ui.current_tab = tab;
                app.activate_menu();
            }
        }
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
    if bound(app, "PICK_BLOCK", &input) {
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
        let facing = app.panels.build.placement_facing.rotate_cw();
        app.panels.build.placement_facing = facing;
        app.core
            .game
            .res
            .log(format!("Placing facing {}", facing.name()));
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

/// Within the keyboard's reach: one of the 8 tiles around the player, on their level.
fn in_reach(app: &App, tile: Position) -> bool {
    app.core.game.get_player_position().is_some_and(|p| {
        p.z == tile.z && (p.x - tile.x).abs() <= 1 && (p.y - tile.y).abs() <= 1 && p != tile
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
    fn too_far_looks_instead() {
        let mut runner = chain_commands(WITH_CHEST);
        click(&mut runner, 4, 0, "left");
        assert_eq!(block(&runner, -11, 0), None);
        assert!(last_message(&runner).ends_with("Too far to build there"));
        assert!(runner.app.panels.look.mode);
        assert_eq!(
            runner.app.panels.look.cursor,
            Position { x: -11, y: 0, z: 0 }
        );
        click(&mut runner, 0, 5, "right");
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
            .find(|&c| tab_at(&runner.app, c) == Some(MenuTab::ItemDictionary))
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
}

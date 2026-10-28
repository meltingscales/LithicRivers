//! Moving up or down where there is nothing to climb: offer to build a stairway there, or
//! say how to get one.

use crate::app_state::StairsPrompt;
use crate::App;
use crossterm::event::KeyCode;
use lithicrivers_core::stairs::{self, StairsError};

/// Called for the up and down keys. Returns true if it handled the key (the climb can't
/// happen as it is), false to let the move go ahead.
pub fn before_climb(app: &mut App, dz: i64) -> bool {
    let game = &mut app.core.game;
    if game.res.player_state.noclip_enabled {
        return false;
    }
    let (Some(player), Some(here)) = (game.get_player_entity(), game.get_player_position()) else {
        return false;
    };
    let there = lithicrivers_core::components::Position {
        z: here.z + dz,
        ..here
    };
    // Stairs on either end, and room to stand at the other level: the climb goes ahead
    let climbable = stairs::climbable(&game.world, &mut game.res, here)
        || stairs::climbable(&game.world, &mut game.res, there);
    if climbable && lithicrivers_core::logistics::walkable(&mut game.res, &game.world, there) {
        return false;
    }
    let way = if dz > 0 { "up" } else { "down" };
    match stairs::check_stairway(&game.world, &mut game.res, here, dz, player) {
        Ok(cost) => {
            let held = game
                .world
                .get::<&lithicrivers_core::components::Inventory>(player)
                .map(|inv| {
                    inv.slots
                        .iter()
                        .filter(|s| s.kind == lithicrivers_core::components::ItemKind::Stairs)
                        .map(|s| s.qty)
                        .sum::<u32>()
                })
                .unwrap_or(0);
            if held >= cost {
                app.panels.stairs_prompt = Some(StairsPrompt {
                    dz,
                    cost,
                    selected: 0,
                });
            } else {
                game.res.log(format!(
                    "Nothing to climb {way} here. Craft {} and try going {way} again to build a stairway",
                    if cost == 1 { "1 stairs".to_string() } else { format!("{cost} stairs") }
                ));
            }
        }
        Err(StairsError::Occupied) => {
            game.res.log(format!(
                "Nothing to climb {way} here, and no room to build stairs"
            ));
        }
        Err(e) => game
            .res
            .log(format!("Nothing to climb {way} here. {}", e.message())),
    }
    true
}

/// The prompt's rows.
pub const CHOICES: [&str; 2] = ["Yes, build it", "No"];

/// Keys while the prompt is open. Returns false if it isn't open.
pub fn handle_stairs_prompt_input(app: &mut App, key: KeyCode) -> bool {
    let Some(mut prompt) = app.panels.stairs_prompt else {
        return false;
    };
    let keys = &app.ui.keybinds;
    if keys.matches("ui", "CLOSE_HELP_MENU", &key) {
        app.panels.stairs_prompt = None;
    } else if keys.menu_up(&key) {
        prompt.selected = 0;
        app.panels.stairs_prompt = Some(prompt);
    } else if keys.menu_down(&key) {
        prompt.selected = 1;
        app.panels.stairs_prompt = Some(prompt);
    } else if keys.matches("ui", "MENU_ACTIVATE", &key) {
        app.panels.stairs_prompt = None;
        if prompt.selected == 0 {
            build_and_climb(app, prompt.dz);
        }
    }
    true
}

fn build_and_climb(app: &mut App, dz: i64) {
    let game = &mut app.core.game;
    let (Some(player), Some(here)) = (game.get_player_entity(), game.get_player_position()) else {
        return;
    };
    match stairs::build_stairway(&mut game.world, &mut game.res, here, dz, player) {
        Ok(()) => {
            game.res.log(format!(
                "Built stairs {} to z = {}",
                if dz > 0 { "up" } else { "down" },
                here.z + dz
            ));
            game.queue_player_move_z(dz);
            let tick_result = game.tick();
            app.apply_tick_result(&tick_result);
            app.snap_view_to_player_z();
        }
        Err(e) => game.res.log(e.message()),
    }
}

#[cfg(test)]
mod tests {
    use crate::headless::Runner;
    use crate::ui::click_targets::ClickTarget;

    fn at_origin_with_stairs() -> Runner {
        let mut runner = Runner::new(12345, (100, 30));
        runner.chain(&[
            "world factory-showcase",
            "key ui.MENU_PREV",
            "cheat tp 0 5 0",
            "cheat giveitem stairs 2",
            "key movement.MOVE_UP",
        ]);
        assert!(runner.app.panels.stairs_prompt.is_some());
        runner
    }

    fn z(runner: &Runner) -> i64 {
        runner.app.core.game.get_player_position().unwrap().z
    }

    #[test]
    fn choosing_no_builds_nothing() {
        let mut runner = at_origin_with_stairs();
        runner.chain(&["key movement.MOVE_SOUTH", "key ui.MENU_ACTIVATE"]);
        assert!(runner.app.panels.stairs_prompt.is_none());
        assert_eq!(z(&runner), 0);
        let tile = runner
            .app
            .core
            .game
            .res
            .world_state
            .world
            .get_tile_cached(0, 5, 0);
        assert_ne!(tile, lithicrivers_core::tiles::TileKind::Stairs);
    }

    #[test]
    fn clicking_yes_builds_and_climbs() {
        let mut runner = at_origin_with_stairs();
        let _ = runner.screen_text();
        let cell = (0..30)
            .flat_map(|row| (0..100).map(move |col| (col, row)))
            .find(|&(c, r)| {
                matches!(
                    runner.app.ui.click_targets.hit(c, r),
                    Some((_, ClickTarget::StairsChoice(0)))
                )
            })
            .expect("the Yes row is on screen");
        runner.chain(&[&format!("click {} {} left", cell.0, cell.1)]);
        assert_eq!(z(&runner), 1);
    }

    #[test]
    fn a_climbable_stack_climbs_without_asking() {
        let mut runner = Runner::new(12345, (100, 30));
        runner.chain(&[
            "world factory-showcase",
            "key ui.MENU_PREV",
            "cheat tp 0 5 0",
        ]);
        for z in 0..2 {
            let game = &mut runner.app.core.game;
            lithicrivers_core::factory::place(
                &mut game.world,
                &mut game.res,
                lithicrivers_core::factory::FactoryKind::VerticalConveyorUp,
                lithicrivers_core::components::Position { x: 1, y: 5, z },
                lithicrivers_core::factory::Facing::North,
            )
            .unwrap();
        }
        runner.chain(&["key movement.MOVE_EAST", "key movement.MOVE_UP"]);
        assert!(runner.app.panels.stairs_prompt.is_none());
        assert_eq!(z(&runner), 1);
    }
}

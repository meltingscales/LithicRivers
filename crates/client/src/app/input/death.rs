//! The system-failure screen: while the robot is dead, the only choices are to choose where
//! to wake (the Resurrect screen) or to save and quit to the title. Every other key is
//! ignored.

use crate::App;
use crossterm::event::KeyCode;
use lithicrivers_core::components::Position;
use lithicrivers_core::death::WakeChoice;
use lithicrivers_core::vat::ReadyVat;

/// The screen's rows.
pub const CHOICES: [&str; 2] = ["Choose where to wake", "Save and quit to title"];

/// Whether the system-failure screen is up: the player is dead in a game being played.
pub fn showing(app: &App) -> bool {
    app.splash.state == crate::SplashState::MainUI
        && app
            .core
            .game
            .get_player_entity()
            .is_some_and(|p| lithicrivers_core::death::is_dead(&app.core.game.world, p))
}

/// Where the dead robot lies, which the vats' distances are measured from.
pub fn wreck_position(app: &App) -> Position {
    app.core
        .game
        .get_player_position()
        .unwrap_or(Position { x: 0, y: 0, z: 0 })
}

/// The Resurrect screen's choices: every ready vat, nearest first. The junker is the row
/// after them.
pub fn ready_vats(app: &App) -> Vec<ReadyVat> {
    lithicrivers_core::vat::ready_vats(&app.core.game.world, wreck_position(app))
}

/// Keys while the robot is dead. Returns false when the screen isn't up.
pub fn handle_death_input(app: &mut App, key: KeyCode) -> bool {
    if !showing(app) {
        return false;
    }
    let keys = &app.ui.keybinds;
    if let Some(selected) = app.panels.resurrect {
        let rows = ready_vats(app).len() + 1;
        if keys.menu_up(&key) {
            app.panels.resurrect = Some(selected.saturating_sub(1));
        } else if keys.menu_down(&key) {
            app.panels.resurrect = Some((selected + 1).min(rows - 1));
        } else if keys.matches("ui", "MENU_ACTIVATE", &key) {
            wake(app, selected);
        } else if keys.matches("ui", "CLOSE_HELP_MENU", &key) {
            app.panels.resurrect = None;
        }
        return true;
    }
    if keys.menu_up(&key) {
        app.panels.death_choice = 0;
    } else if keys.menu_down(&key) {
        app.panels.death_choice = 1;
    } else if keys.matches("ui", "MENU_ACTIVATE", &key) {
        choose(app);
    }
    true
}

pub(crate) fn choose(app: &mut App) {
    if app.panels.death_choice == 0 {
        app.panels.resurrect = Some(0);
    } else if let Err(e) = app.save_and_quit_to_title() {
        app.core.game.res.log_red(format!("Save failed: {e:#}"));
    }
}

/// Wake in the Resurrect screen's row `row`: a ready vat, or the junker after them. Then
/// fly the view from the wreck to where the robot woke.
pub(crate) fn wake(app: &mut App, row: usize) {
    let vats = ready_vats(app);
    let choice = vats
        .get(row)
        .map_or(WakeChoice::Junker, |v| WakeChoice::Vat(v.entity));
    let wreck = wreck_position(app);
    app.combat = crate::app_state::CombatUiState::None;
    if let Some(woke_at) = app.core.game.resurrect(choice) {
        app.panels.resurrect = None;
        app.ui.current_tab = crate::MenuTab::World;
        // One tick so the light and fog of war catch up with the new position
        let tick_result = app.core.game.tick();
        app.apply_tick_result(&tick_result);
        app.start_flyover(wreck, woke_at);
    }
}

#[cfg(test)]
mod tests {
    use crate::headless::{chain_commands, Runner};
    use crate::ui::click_targets::ClickTarget;
    use lithicrivers_core::components::{Inventory, ItemKind, Position, Wreck};
    use lithicrivers_core::factory::{self, Facing, FactoryKind};
    use lithicrivers_core::resources::QuestMarkerType;

    /// Choose where to wake, take the junker body, and let the flyover play out.
    const WAKE_IN_JUNKER: [&str; 3] = ["key ui.MENU_ACTIVATE", "key ui.MENU_ACTIVATE", "wait 3000"];

    fn dead() -> Runner {
        let mut runner = Runner::new(12345, (100, 30));
        runner.chain(&[
            "skip-splash",
            "key ui.MENU_PREV",
            "cheat tp 0 0 0",
            "cheat giveitem chest 2",
            "cheat kill_player",
        ]);
        assert!(super::showing(&runner.app));
        runner
    }

    fn held(runner: &Runner, kind: ItemKind) -> u32 {
        runner
            .app
            .core
            .game
            .get_player_component::<Inventory>()
            .map(|inv| {
                inv.slots
                    .iter()
                    .filter(|s| s.kind == kind)
                    .map(|s| s.qty)
                    .sum()
            })
            .unwrap_or(0)
    }

    #[test]
    fn respawning_then_looting_the_wreck_gets_everything_back() {
        let mut runner = dead();
        runner.chain(&WAKE_IN_JUNKER);
        assert!(!super::showing(&runner.app));
        assert_eq!(
            held(&runner, ItemKind::Chest),
            0,
            "the chests stayed with the wreck"
        );
        let wreck = runner
            .app
            .core
            .game
            .world
            .query::<&Position>()
            .with::<&Wreck>()
            .iter()
            .map(|(_, p)| *p)
            .next()
            .expect("a wreck");
        assert_eq!(wreck, Position { x: 0, y: 0, z: 0 });
        // Walk back (by teleport) and take everything
        runner.chain(&["cheat tp 1 0 0", "key action.INTERACT"]);
        runner.chain(&["key ui.TAKE_ALL"]);
        assert_eq!(held(&runner, ItemKind::Chest), 2);
    }

    #[test]
    fn deleting_the_wreck_marker_keeps_the_wreck_and_looting_it_removes_the_marker() {
        let wreck_markers = |runner: &Runner| {
            runner
                .app
                .core
                .game
                .res
                .quest_markers
                .iter()
                .filter(|m| m.marker_type == QuestMarkerType::Death)
                .count()
        };
        let mut runner = dead();
        runner.chain(&WAKE_IN_JUNKER);
        assert_eq!(wreck_markers(&runner), 1);

        // Looting it empty takes the marker away
        runner.chain(&["cheat tp 1 0 0", "key action.INTERACT", "key ui.TAKE_ALL"]);
        runner.chain(&["key ui.CLOSE_HELP_MENU", "key movement.WAIT"]);
        assert_eq!(wreck_markers(&runner), 0);

        // A second death, whose marker is deleted by hand
        runner.chain(&["cheat kill_player"]);
        runner.chain(&WAKE_IN_JUNKER);
        runner.chain(&["key ui.MENU_NEXT 2"]);
        // Markers are listed nearest first, and the junker woke at the HQ, by the factory
        runner.chain(&["rawkey DOWN", "rawkey DOWN", "rawkey DOWN"]);
        runner.chain(&["expect-text \"Wreck 2\""]);
        runner.chain(&[
            "key global_map.DELETE_MARKER",
            "key global_map.CONFIRM_DELETE",
        ]);
        assert_eq!(wreck_markers(&runner), 0);
        let wreck_items: u32 = runner
            .app
            .core
            .game
            .world
            .query::<&Inventory>()
            .with::<&Wreck>()
            .iter()
            .map(|(_, inv)| inv.slots.iter().map(|s| s.qty).sum::<u32>())
            .sum();
        assert!(wreck_items > 0, "the wreck keeps its items");
    }

    #[test]
    fn an_empty_wreck_can_be_cleared_away() {
        let wreck_count =
            |runner: &Runner| runner.app.core.game.world.query::<&Wreck>().iter().count();
        let mut runner = dead();
        runner.chain(&WAKE_IN_JUNKER);
        runner.chain(&["cheat tp 1 0 0"]);
        runner.chain(&["key action.INTERACT", "key ui.TAKE_ALL"]);
        runner.chain(&["key ui.CLOSE_HELP_MENU"]);
        assert_eq!(wreck_count(&runner), 1, "looting leaves the empty wreck");
        runner.chain(&[
            "key action.INTERACT",
            "expect-text \"You clear away the empty wreck.\"",
        ]);
        assert_eq!(wreck_count(&runner), 0);
    }

    #[test]
    fn keep_inventory_and_keep_body_survive_a_reload_and_apply_on_death() {
        let mut runner = chain_commands(&["skip-splash", "cheat keepinventory"]);
        runner.chain(&["expect-text \"Keep inventory: off\""]);
        runner.chain(&[
            "cheat keepinventory on",
            "cheat keepbody on",
            "save-reload",
            "cheat keepbody",
            "expect-text \"Keep body: on\"",
        ]);
        let settings = runner.app.core.game.res.world_settings;
        assert!(settings.keep_inventory && settings.keep_body);
        let before = held(&runner, ItemKind::Torch);
        assert!(before > 0, "the starting kit has torches");
        runner.chain(&["cheat kill_player"]);
        runner.chain(&WAKE_IN_JUNKER);
        assert_eq!(held(&runner, ItemKind::Torch), before);

        runner.chain(&[
            "cheat keepinventory off",
            "expect-text \"Keep inventory: off\"",
        ]);
        assert!(!runner.app.core.game.res.world_settings.keep_inventory);
        let script = crate::headless::parse_script("cheat keepbody maybe").unwrap();
        let error = runner.run_steps(&script.steps).error.unwrap_or_default();
        assert!(error.contains("Usage: keepbody [on|off]"), "{error}");
    }

    /// The screen cell of the first click target `wanted` accepts.
    fn cell_of(runner: &mut Runner, wanted: impl Fn(&ClickTarget) -> bool) -> (u16, u16) {
        let _ = runner.screen_text();
        (0..30)
            .flat_map(|row| (0..100).map(move |col| (col, row)))
            .find(|&(c, r)| {
                runner
                    .app
                    .ui
                    .click_targets
                    .hit(c, r)
                    .is_some_and(|(_, t)| wanted(&t))
            })
            .expect("the row is on screen")
    }

    #[test]
    fn clicking_respawn_then_the_junker_wakes_and_quitting_saves_to_the_title() {
        let mut runner = dead();
        let cell = cell_of(&mut runner, |t| matches!(t, ClickTarget::DeathChoice(0)));
        runner.chain(&[&format!("click {} {} left", cell.0, cell.1)]);
        assert_eq!(runner.app.panels.resurrect, Some(0));
        let cell = cell_of(&mut runner, |t| {
            matches!(t, ClickTarget::ResurrectChoice(0))
        });
        runner.chain(&[&format!("click {} {} left", cell.0, cell.1)]);
        assert!(!super::showing(&runner.app));

        let mut runner = dead();
        runner.chain(&["key movement.MOVE_SOUTH", "key ui.MENU_ACTIVATE"]);
        assert_eq!(runner.app.splash.state, crate::SplashState::Title);
    }

    #[test]
    fn back_leaves_the_resurrect_screen_for_the_system_failure_screen() {
        let mut runner = dead();
        runner.chain(&["key ui.MENU_ACTIVATE", "expect-text \"RESURRECT\""]);
        runner.chain(&["key ui.CLOSE_HELP_MENU", "expect-text \"SYSTEM FAILURE\""]);
        assert!(super::showing(&runner.app));
    }

    /// Two crude vats east and west of spawn, each with a torso and a fusion core, and the
    /// eastern one with a head too.
    fn dead_with_two_vats() -> Runner {
        let mut runner = Runner::new(12345, (100, 30));
        runner.chain(&["skip-splash", "key ui.MENU_PREV", "cheat tp 0 0 0"]);
        let game = &mut runner.app.core.game;
        for (x, head) in [(3, true), (-6, false)] {
            let at = Position { x, y: 0, z: 0 };
            game.res.world_state.world.set_tile_cached(
                x,
                0,
                0,
                lithicrivers_core::tiles::TileKind::Grass,
            );
            game.res.world_state.world.set_tile_cached(
                x + 1,
                0,
                0,
                lithicrivers_core::tiles::TileKind::Grass,
            );
            let vat = factory::place(
                &mut game.world,
                &mut game.res,
                FactoryKind::CrudeVat,
                at,
                Facing::North,
            )
            .unwrap();
            let mut parts = vec![ItemKind::CrudeTorso, ItemKind::CrudeFusionCore];
            if head {
                parts.push(ItemKind::CrudeHead);
            }
            for item in parts {
                assert!(factory::insert(&mut game.world, vat, item));
            }
        }
        runner.chain(&["cheat kill_player", "key ui.MENU_ACTIVATE"]);
        runner
    }

    #[test]
    fn the_resurrect_screen_lists_both_vats_nearest_first_and_the_junker() {
        let mut runner = dead_with_two_vats();
        let screen = runner.screen_text();
        let east = screen.find("Crude resurrection vat at (3, 0, 0), 3 tiles");
        let west = screen.find("Crude resurrection vat at (-6, 0, 0), 6 tiles");
        let junker = screen.find("Junker body at the SapienCorp HQ");
        assert!(
            east.is_some() && west.is_some() && junker.is_some(),
            "{screen}"
        );
        assert!(east < west && west < junker, "{screen}");
        assert!(screen.contains("Damaged: head, torso, fusion core. Missing: arms, legs"));
    }

    #[test]
    fn choosing_the_second_vat_wakes_beside_it_and_empties_it() {
        let mut runner = dead_with_two_vats();
        runner.chain(&["key ui.MENU_DOWN", "key ui.MENU_ACTIVATE", "wait 3000"]);
        assert!(!super::showing(&runner.app));
        let player = runner.app.core.game.get_player_position().unwrap();
        assert_eq!(
            player,
            Position { x: -5, y: 0, z: 0 },
            "east of the western vat"
        );
        let body = runner
            .app
            .core
            .game
            .get_player_component::<lithicrivers_core::model::body::Body>()
            .unwrap()
            .clone();
        use lithicrivers_core::model::body::{BodyPartState, BodyPartType};
        assert_eq!(
            body.parts[&BodyPartType::Head].state,
            BodyPartState::Missing
        );
        assert_eq!(
            body.parts[&BodyPartType::Torso].state,
            BodyPartState::Damaged
        );
        let vats: Vec<usize> = runner
            .app
            .core
            .game
            .world
            .query::<(&Inventory, &factory::FactoryBlock)>()
            .iter()
            .map(|(_, (inv, _))| inv.slots.len())
            .collect();
        assert!(vats.contains(&0) && vats.contains(&3), "{vats:?}");
    }

    #[test]
    fn the_flyover_ends_on_the_destination_at_the_largest_scale_and_a_key_skips_it() {
        let finished = |skip: bool| {
            let mut runner = dead_with_two_vats();
            runner.chain(&["key ui.MENU_ACTIVATE", "wait 1000"]);
            assert!(runner.app.flyover.is_some(), "still flying");
            if skip {
                runner.chain(&["key movement.WAIT"]);
            } else {
                runner.chain(&["wait 2500"]);
            }
            assert!(runner.app.flyover.is_none());
            let ui = &runner.app.ui;
            assert_eq!(ui.scale, crate::sprite_loader::Scale::Large);
            assert_eq!((ui.view_x, ui.view_y, ui.view_z), (4, 0, 0));
            crate::headless::state_json(&runner.app)
        };
        assert_eq!(finished(true), finished(false), "skipping changes nothing");
    }

    #[test]
    fn a_save_made_while_dead_shows_the_screen_again() {
        let mut runner = dead();
        runner.chain(&["save-reload"]);
        assert!(super::showing(&runner.app));
        assert!(runner.screen_text().contains("self-destruct command"));
    }
}

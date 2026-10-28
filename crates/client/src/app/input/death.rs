//! The system-failure screen: while the robot is dead, the only choices are to respawn at
//! the bunker or to save and quit to the title. Every other key is ignored.

use crate::App;
use crossterm::event::KeyCode;

/// The screen's rows.
pub const CHOICES: [&str; 2] = ["Respawn at the bunker", "Save and quit to title"];

/// Whether the system-failure screen is up: the player is dead in a game being played.
pub fn showing(app: &App) -> bool {
    app.splash.state == crate::SplashState::MainUI
        && app
            .core
            .game
            .get_player_entity()
            .is_some_and(|p| lithicrivers_core::death::is_dead(&app.core.game.world, p))
}

/// Keys while the robot is dead. Returns false when the screen isn't up.
pub fn handle_death_input(app: &mut App, key: KeyCode) -> bool {
    if !showing(app) {
        return false;
    }
    let keys = &app.ui.keybinds;
    if keys.matches("movement", "MOVE_NORTH", &key) || keys.matches("ui", "CREDITS_SCROLL_UP", &key)
    {
        app.panels.death_choice = 0;
    } else if keys.matches("movement", "MOVE_SOUTH", &key)
        || keys.matches("ui", "CREDITS_SCROLL_DOWN", &key)
    {
        app.panels.death_choice = 1;
    } else if keys.matches("ui", "MENU_ACTIVATE", &key) {
        choose(app);
    }
    true
}

fn choose(app: &mut App) {
    if app.panels.death_choice == 0 {
        app.combat = crate::app_state::CombatUiState::None;
        if app.core.game.respawn().is_some() {
            app.ui.current_tab = crate::MenuTab::World;
            // One tick so the light and fog of war catch up with the new position
            let tick_result = app.core.game.tick();
            app.apply_tick_result(&tick_result);
            app.snap_view_to_player_z();
        }
    } else if let Err(e) = app.save_and_quit_to_title() {
        app.core.game.res.log_red(format!("Save failed: {e:#}"));
    }
}

#[cfg(test)]
mod tests {
    use crate::headless::{chain_commands, Runner};
    use crate::ui::click_targets::ClickTarget;
    use lithicrivers_core::components::{Inventory, ItemKind, Position, Wreck};
    use lithicrivers_core::resources::QuestMarkerType;

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
        runner.chain(&["key ui.MENU_ACTIVATE"]);
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
        runner.chain(&["key ui.MENU_ACTIVATE"]);
        assert_eq!(wreck_markers(&runner), 1);

        // Looting it empty takes the marker away
        runner.chain(&["cheat tp 1 0 0", "key action.INTERACT", "key ui.TAKE_ALL"]);
        runner.chain(&["key ui.CLOSE_HELP_MENU", "key movement.WAIT"]);
        assert_eq!(wreck_markers(&runner), 0);

        // A second death, whose marker is deleted by hand
        runner.chain(&[
            "cheat kill_player",
            "key ui.MENU_ACTIVATE",
            "key ui.MENU_NEXT 2",
        ]);
        runner.chain(&["rawkey DOWN", "rawkey DOWN", "expect-text \"Wreck 2\""]);
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
        runner.chain(&["key ui.MENU_ACTIVATE", "cheat tp 1 0 0"]);
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
        runner.chain(&["cheat kill_player", "key ui.MENU_ACTIVATE"]);
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

    #[test]
    fn clicking_respawn_works_and_quitting_saves_to_the_title() {
        let mut runner = dead();
        let _ = runner.screen_text();
        let cell = (0..30)
            .flat_map(|row| (0..100).map(move |col| (col, row)))
            .find(|&(c, r)| {
                matches!(
                    runner.app.ui.click_targets.hit(c, r),
                    Some((_, ClickTarget::DeathChoice(0)))
                )
            })
            .expect("the respawn row is on screen");
        runner.chain(&[&format!("click {} {} left", cell.0, cell.1)]);
        assert!(!super::showing(&runner.app));

        let mut runner = dead();
        runner.chain(&["key movement.MOVE_SOUTH", "key ui.MENU_ACTIVATE"]);
        assert_eq!(runner.app.splash.state, crate::SplashState::Title);
    }

    #[test]
    fn a_save_made_while_dead_shows_the_screen_again() {
        let mut runner = dead();
        runner.chain(&["save-reload"]);
        assert!(super::showing(&runner.app));
        assert!(runner.screen_text().contains("self-destruct command"));
    }
}

//! Saving and loading must not change anything: every committed script gives the same
//! output with a `save-reload` step inserted anywhere, and scenarios with state that saves
//! used to drop (combat, tutorial progress, hotbar) survive a load into a fresh client.

use crate::headless::{
    chain_commands, parse_script, Runner, Script, ScriptLine, Step, DEFAULT_SIZE, EXIT_OK,
};
use std::path::{Path, PathBuf};

/// Insert `save-reload` at most this many times per script, spread evenly.
const MAX_RELOAD_POINTS: usize = 40;

fn repo_scripts() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts");
    let mut paths = vec![root.join("agent/smoke.txt")];
    for dir in ["agent/bugs", "snapshots"] {
        let mut found: Vec<PathBuf> = std::fs::read_dir(root.join(dir))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "txt"))
            .collect();
        found.sort();
        paths.extend(found);
    }
    paths
}

/// Exit status and output; a panic counts as status -1 with the panic message as output.
fn run(script: &Script, steps: &[ScriptLine]) -> (i32, String) {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut runner = Runner::new(
            script.seed.unwrap_or(0),
            script.size.unwrap_or(DEFAULT_SIZE),
        );
        let report = runner.run_steps(steps);
        (report.status, report.output)
    }))
    .unwrap_or_else(|panic| {
        let message = panic
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default();
        (-1, format!("panic: {message}"))
    })
}

fn first_difference(a: &str, b: &str) -> String {
    let (a_lines, b_lines): (Vec<&str>, Vec<&str>) = (a.lines().collect(), b.lines().collect());
    let i = a_lines
        .iter()
        .zip(&b_lines)
        .position(|(x, y)| x != y)
        .unwrap_or(a_lines.len().min(b_lines.len()));
    format!(
        "output line {}: expected {:?}, got {:?}",
        i + 1,
        a_lines.get(i).unwrap_or(&"<end>"),
        b_lines.get(i).unwrap_or(&"<end>")
    )
}

#[test]
fn save_reload_anywhere_changes_nothing() {
    let mut failures = Vec::new();
    for path in repo_scripts() {
        let text = std::fs::read_to_string(&path).unwrap();
        let script = parse_script(&text).unwrap();
        let expected = run(&script, &script.steps);
        assert_eq!(expected.0, EXIT_OK, "{} fails on its own", path.display());

        let n = script.steps.len();
        let stride = n.div_ceil(MAX_RELOAD_POINTS).max(1);
        for at in (0..=n).step_by(stride) {
            let mut steps = script.steps.clone();
            let line = steps.get(at.saturating_sub(1)).map_or(0, |l| l.line);
            steps.insert(
                at,
                ScriptLine {
                    line,
                    step: Step::SaveReload,
                },
            );
            let got = run(&script, &steps);
            if got != expected {
                let name = path.file_name().unwrap().to_string_lossy();
                let detail = if got.0 != expected.0 {
                    let first = got.1.lines().next().unwrap_or_default();
                    format!("status {} instead of {}: {first}", got.0, expected.0)
                } else {
                    first_difference(&expected.1, &got.1)
                };
                failures.push(format!("{name}, reload after line {line}: {detail}"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} reload points changed the output:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// --- Loading into a fresh client, as the Menu tab's Load does -----------------------

/// A new client (past the splash) with `saved` loaded into it.
fn fresh_runner_with(saved: &str) -> Runner {
    let mut runner = chain_commands(&["skip-splash"]);
    runner.app.load_from_str(saved).expect("load");
    runner
}

/// The player's queued combat moves as (move name, target).
fn player_queue(runner: &Runner) -> Vec<(String, Option<hecs::Entity>)> {
    use lithicrivers_core::moves::{ActionQueue, CombatAction};
    let game = &runner.app.core.game;
    let player = game.get_player_entity().unwrap();
    let queue = game
        .world
        .get::<&ActionQueue>(player)
        .expect("player has a queue");
    queue
        .current_action
        .iter()
        .chain(queue.actions.iter())
        .map(|queued| match &queued.action {
            CombatAction::PlayerMove {
                move_data,
                target_entity,
                ..
            } => (move_data.name.clone(), *target_entity),
            CombatAction::EnemyAttack { target_entity, .. } => {
                ("enemy attack".into(), Some(*target_entity))
            }
        })
        .collect()
}

const MID_COMBAT: &[&str] = &[
    "skip-splash",
    "cheat tp 0 0 0",
    "cheat spawn_enemy feral_dog",
    "key movement.WAIT",
    "key ui.MENU_ACTIVATE",
];

#[test]
fn loading_mid_combat_restores_combat_and_plays_on_identically() {
    let mut original = chain_commands(MID_COMBAT);
    let queue = player_queue(&original);
    assert!(!queue.is_empty(), "the scenario queues an attack");
    let saved = original.app.save_to_string().expect("save");

    let mut loaded = fresh_runner_with(&saved);
    assert!(loaded.app.core.game.res.player_state.combat_active);
    assert!(
        loaded.app.combat.is_active(),
        "combat panel reopens on load"
    );
    assert_eq!(player_queue(&loaded), queue, "same moves, same targets");

    let next = &[
        "tick 3",
        "state",
        "screen",
        "key ui.MENU_ACTIVATE",
        "tick 5",
        "state",
        "screen",
    ];
    let (expected, got) = (original.chain(next), loaded.chain(next));
    assert!(expected == got, "{}", first_difference(&expected, &got));
}

#[test]
fn queued_attack_still_targets_the_same_dog() {
    let original = chain_commands(MID_COMBAT);
    let target = player_queue(&original)[0].1.expect("melee has a target");
    let dog_pos = *original
        .app
        .core
        .game
        .world
        .get::<&lithicrivers_core::components::Position>(target)
        .unwrap();

    let loaded = fresh_runner_with(&original.app.save_to_string().unwrap());
    assert_eq!(player_queue(&loaded)[0].1, Some(target));
    let loaded_pos = *loaded
        .app
        .core
        .game
        .world
        .get::<&lithicrivers_core::components::Position>(target)
        .expect("target entity exists after load");
    assert_eq!(loaded_pos, dog_pos);
}

#[test]
fn tutorial_progress_survives_a_load() {
    let mut original = chain_commands(&["skip-splash"]);
    let tutorial = &mut original.app.ui.tutorial_system;
    tutorial.current_step_index = 3;
    tutorial.completed_tutorials = vec!["a".into(), "b".into(), "c".into()];

    let loaded = fresh_runner_with(&original.app.save_to_string().unwrap());
    let tutorial = &loaded.app.ui.tutorial_system;
    assert_eq!(tutorial.current_step_index, 3);
    assert_eq!(tutorial.completed_tutorials, ["a", "b", "c"]);
}

#[test]
fn hotbar_assignments_survive_a_load() {
    use lithicrivers_core::components::ItemKind;
    let mut original = chain_commands(&["skip-splash"]);
    original.app.panels.build.hotbar_assignments[0] = Some(ItemKind::Log);
    original.app.panels.build.hotbar_assignments[4] = Some(ItemKind::Stick);

    let loaded = fresh_runner_with(&original.app.save_to_string().unwrap());
    assert_eq!(
        loaded.app.panels.build.hotbar_assignments,
        original.app.panels.build.hotbar_assignments
    );
}

#[test]
fn the_craftable_only_filter_survives_a_load() {
    let original = chain_commands(&[
        "skip-splash",
        "key ui.MENU_NEXT 5",
        "key crafting.TOGGLE_CRAFTABLE_ONLY",
    ]);
    assert!(original.app.panels.crafting.craftable_only);
    let loaded = fresh_runner_with(&original.app.save_to_string().unwrap());
    assert!(loaded.app.panels.crafting.craftable_only);

    // A save from before the filter loads with it off
    let mut save: serde_json::Value =
        serde_json::from_str(&original.app.save_to_string().unwrap()).unwrap();
    save["client"]
        .as_object_mut()
        .unwrap()
        .remove("craftable_only");
    let old = fresh_runner_with(&save.to_string());
    assert!(!old.app.panels.crafting.craftable_only);
}

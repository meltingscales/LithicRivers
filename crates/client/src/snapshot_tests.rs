//! Snapshot tests: every `scripts/snapshots/*.txt` runs headless and its full output is
//! compared with a stored snapshot in `src/snapshots/`. Review changes with
//! `cargo insta review`, or re-record with `INSTA_UPDATE=always cargo test`.

use crate::headless::{run_script, Overrides, EXIT_OK};

/// Text that depends on the build or machine rather than the game: masked so snapshots
/// survive commits and match on every OS.
const BUILD_INFO_FILTERS: &[(&str, &str)] = &[
    (r"(Version|Git Branch|Git Commit): [^│\n]*", "$1: <build>"),
    (r"External file: [^│\n]*", "External file: <config path>"),
];

#[test]
fn script_snapshots() {
    let mut settings = insta::Settings::clone_current();
    settings.set_prepend_module_to_snapshot(false);
    for (pattern, replacement) in BUILD_INFO_FILTERS {
        settings.add_filter(pattern, *replacement);
    }

    settings.bind(|| {
        // Runs every script, then reports all mismatches together
        insta::glob!("../../../scripts/snapshots", "*.txt", |path| {
            let text = std::fs::read_to_string(path).unwrap();
            let (mut out, mut err_out) = (Vec::new(), Vec::new());
            let code = run_script(&text, 0, Overrides::default(), &mut out, &mut err_out);
            assert_eq!(
                code,
                EXIT_OK,
                "snapshot script {} failed:\n{}",
                path.display(),
                String::from_utf8_lossy(&err_out)
            );
            insta::assert_snapshot!(String::from_utf8(out).unwrap());
        });
    });
}

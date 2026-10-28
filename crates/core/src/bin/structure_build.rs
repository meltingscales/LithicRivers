//! `just util-structure-build`: compile every structure's `plan.toml` into its
//! `shape_layers.txt` and `data.json` legend. Structures without a plan are hand-written
//! and left alone.

use lithicrivers_core::structure_plan::build_folder;

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../client/assets/structures");
    let mut dirs: Vec<_> = std::fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("read {}: {e}", root.display()))
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.join("plan.toml").exists())
        .collect();
    dirs.sort();
    let mut failed = false;
    for dir in dirs {
        let name = dir.file_name().unwrap_or_default().to_string_lossy();
        match build_folder(&dir) {
            Ok(true) => println!("built {name}"),
            Ok(false) => println!("{name} is up to date"),
            Err(e) => {
                eprintln!("error: {e}");
                failed = true;
            }
        }
    }
    if failed {
        std::process::exit(1);
    }
}

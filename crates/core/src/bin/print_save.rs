//! Print a world's save as readable JSON: `print_save <world folder or save file>`.
//! Saves are gzipped; this is for debugging and for reading them by eye.

use std::path::{Path, PathBuf};

fn main() -> anyhow::Result<()> {
    let arg = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("usage: print_save <world folder or save file>"))?;
    let path = PathBuf::from(arg);
    let file = if path.is_dir() {
        ["world.json.gz", "world.json"]
            .into_iter()
            .map(|name| path.join(name))
            .find(|p| p.is_file())
            .ok_or_else(|| anyhow::anyhow!("no world.json.gz or world.json in {path:?}"))?
    } else {
        path
    };
    let json = lithicrivers_core::save_load::read_save_json(Path::new(&file))?;
    let value: serde_json::Value = serde_json::from_str(&json)?;
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

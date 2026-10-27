//! Worlds on disk: each world is a folder under the saves folder holding `world.json` (the
//! game save) and `meta.json` (name, seed, settings and dates, read by the world list).

use anyhow::{bail, Context, Result};
use lithicrivers_core::resources::WorldSettings;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const META_FILE: &str = "meta.json";
pub const SAVE_FILE: &str = "world.json";
const META_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldMeta {
    pub version: u32,
    pub name: String,
    pub seed: u64,
    #[serde(flatten)]
    pub settings: WorldSettings,
    pub created_unix: u64,
    pub last_played_unix: u64,
}

impl WorldMeta {
    pub fn new(name: &str, seed: u64, settings: WorldSettings, now_unix: u64) -> Self {
        Self {
            version: META_VERSION,
            name: name.to_string(),
            seed,
            settings,
            created_unix: now_unix,
            last_played_unix: now_unix,
        }
    }
}

/// A world folder found in the saves folder. `meta` is `None` when its files are missing or
/// unreadable: such a world can be deleted but not played.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldEntry {
    pub folder: PathBuf,
    pub meta: Option<WorldMeta>,
}

/// Turn the seed field's text into a seed: blank picks a new one from the time, a number is
/// used as-is, and any other text is hashed (FNV-1a, so the same text is the same world on
/// every build and platform).
pub fn parse_seed(text: &str, now_unix: u64) -> u64 {
    let text = text.trim();
    if text.is_empty() {
        return now_unix;
    }
    text.parse().unwrap_or_else(|_| fnv1a_64(text.as_bytes()))
}

fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// A new folder name for `name`: lowercase, runs of other characters become `-`, with `-2`,
/// `-3`, ... added if the folder already exists.
pub fn folder_name_for(saves: &Path, name: &str) -> String {
    let mut slug = String::new();
    for c in name.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_matches('-');
    let base = if slug.is_empty() { "world" } else { slug };
    let mut candidate = base.to_string();
    let mut n = 2;
    while saves.join(&candidate).exists() {
        candidate = format!("{base}-{n}");
        n += 1;
    }
    candidate
}

pub fn read_meta(folder: &Path) -> Result<WorldMeta> {
    let path = folder.join(META_FILE);
    let text = std::fs::read_to_string(&path).with_context(|| format!("read {path:?}"))?;
    serde_json::from_str(&text).with_context(|| format!("parse {path:?}"))
}

pub fn write_meta(folder: &Path, meta: &WorldMeta) -> Result<()> {
    let path = folder.join(META_FILE);
    std::fs::write(&path, serde_json::to_string_pretty(meta)?)
        .with_context(|| format!("write {path:?}"))
}

/// Make the folder for a new world and write its metadata.
pub fn create_world_folder(saves: &Path, meta: &WorldMeta) -> Result<PathBuf> {
    let folder = saves.join(folder_name_for(saves, &meta.name));
    std::fs::create_dir_all(&folder).with_context(|| format!("create {folder:?}"))?;
    write_meta(&folder, meta)?;
    Ok(folder)
}

/// Change a world's display name. Its folder keeps its name, so nothing on disk moves.
pub fn rename_world(folder: &Path, name: &str) -> Result<()> {
    let mut meta = read_meta(folder)?;
    meta.name = name.to_string();
    write_meta(folder, &meta)
}

/// Every world folder, most recently played first; unreadable ones last.
pub fn list_worlds(saves: &Path) -> Vec<WorldEntry> {
    let Ok(dir) = std::fs::read_dir(saves) else {
        return Vec::new();
    };
    let mut worlds: Vec<WorldEntry> = dir
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .map(|folder| {
            let meta = read_meta(&folder)
                .ok()
                .filter(|_| folder.join(SAVE_FILE).is_file());
            WorldEntry { folder, meta }
        })
        .collect();
    worlds.sort_by(|a, b| {
        let played = |w: &WorldEntry| w.meta.as_ref().map(|m| m.last_played_unix);
        played(b)
            .cmp(&played(a))
            .then_with(|| a.folder.cmp(&b.folder))
    });
    worlds
}

/// Remove a world folder. Refuses anything that isn't a folder directly inside `saves`.
pub fn delete_world(saves: &Path, folder: &Path) -> Result<()> {
    let saves = saves
        .canonicalize()
        .with_context(|| format!("saves folder {saves:?}"))?;
    let folder = folder
        .canonicalize()
        .with_context(|| format!("world folder {folder:?}"))?;
    if folder.parent() != Some(saves.as_path()) {
        bail!("refusing to delete {folder:?}: not a world in {saves:?}");
    }
    std::fs::remove_dir_all(&folder).with_context(|| format!("delete {folder:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SETTINGS: WorldSettings = WorldSettings {
        cheats_allowed: false,
        developer_mode: true,
        show_tutorial: false,
        world_type: lithicrivers_core::resources::WorldType::Normal,
        keep_inventory: false,
        keep_body: false,
    };

    fn temp_saves(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lr-worlds-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn make_world(saves: &Path, name: &str, last_played: u64) -> PathBuf {
        let folder =
            create_world_folder(saves, &WorldMeta::new(name, 1, SETTINGS, last_played)).unwrap();
        std::fs::write(folder.join(SAVE_FILE), "{}").unwrap();
        folder
    }

    #[test]
    fn seeds_from_text() {
        assert_eq!(parse_seed("12345", 99), 12345);
        assert_eq!(parse_seed("  ", 99), 99);
        assert_eq!(parse_seed("lithic", 99), parse_seed("lithic", 5));
        assert_ne!(parse_seed("lithic", 99), parse_seed("rivers", 99));
        // FNV-1a reference value, fixed across builds and platforms
        assert_eq!(fnv1a_64(b"a"), 0xaf63_dc4c_8601_ec8c);
    }

    #[test]
    fn folder_names_are_slugs_with_collision_suffixes() {
        let saves = temp_saves("names");
        assert_eq!(folder_name_for(&saves, "My World!"), "my-world");
        assert_eq!(folder_name_for(&saves, "***"), "world");
        make_world(&saves, "New World", 0);
        make_world(&saves, "New World", 0);
        assert_eq!(folder_name_for(&saves, "New World"), "new-world-3");
        assert!(saves.join("new-world-2").is_dir());
        std::fs::remove_dir_all(&saves).unwrap();
    }

    #[test]
    fn listing_is_most_recent_first_with_unreadable_worlds_flagged() {
        let saves = temp_saves("list");
        let old = make_world(&saves, "B", 100);
        let new = make_world(&saves, "A", 200);
        let broken = saves.join("broken");
        std::fs::create_dir_all(&broken).unwrap();
        std::fs::write(broken.join(META_FILE), "not json").unwrap();

        let worlds = list_worlds(&saves);
        let folders: Vec<&PathBuf> = worlds.iter().map(|w| &w.folder).collect();
        assert_eq!(folders, [&new, &old, &broken]);
        assert_eq!(worlds[0].meta.as_ref().unwrap().settings, SETTINGS);
        assert!(worlds[2].meta.is_none());
        assert!(list_worlds(&saves.join("missing")).is_empty());
        std::fs::remove_dir_all(&saves).unwrap();
    }

    #[test]
    fn rename_changes_only_the_name() {
        let saves = temp_saves("rename");
        let folder = make_world(&saves, "Old", 7);
        rename_world(&folder, "Shiny New").unwrap();
        let meta = read_meta(&folder).unwrap();
        assert_eq!(meta.name, "Shiny New");
        assert_eq!(
            meta,
            WorldMeta {
                name: "Shiny New".into(),
                ..WorldMeta::new("Old", 1, SETTINGS, 7)
            }
        );
        assert_eq!(list_worlds(&saves)[0].folder, folder, "folder unchanged");
        std::fs::remove_dir_all(&saves).unwrap();
    }

    #[test]
    fn delete_removes_only_world_folders() {
        let saves = temp_saves("delete");
        let world = make_world(&saves, "Doomed", 0);
        assert!(
            delete_world(&saves, &saves).is_err(),
            "not the saves folder"
        );
        assert!(delete_world(&saves, &saves.join("..")).is_err());
        delete_world(&saves, &world).unwrap();
        assert!(!world.exists());
        std::fs::remove_dir_all(&saves).unwrap();
    }
}

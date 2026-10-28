//! Structure plans: a `plan.toml` in a `.lrstructure` folder describes a structure with a
//! few shapes instead of every tile, and compiles into its `shape_layers.txt`.
//!
//! ```toml
//! layers = 2
//! width = 9
//! height = 5
//! background = "."          # tiles no shape sets (default "." = existing worldgen)
//!
//! [legend]                  # merged into data.json's blocks
//! "#" = "rock"
//! " " = "air"
//!
//! [[shape]]                 # shapes draw in file order, later over earlier
//! kind = "room"
//! layer = 0
//! x = 0
//! y = 0
//! w = 5
//! h = 5
//! doors = [[4, 2]]
//! ```
//!
//! Coordinates are `x, y` from the top-left; `layer` counts from the bottom, as in
//! `shape_layers.txt`. See `docs/STRUCTURES.md`.

use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub layers: usize,
    pub width: usize,
    pub height: usize,
    #[serde(default = "default_background")]
    pub background: char,
    #[serde(default)]
    pub legend: BTreeMap<String, String>,
    #[serde(default)]
    pub shape: Vec<Shape>,
}

fn default_background() -> char {
    '.'
}
fn wall() -> char {
    '#'
}
fn floor() -> char {
    ' '
}
fn door() -> char {
    'D'
}
fn stairs() -> char {
    '>'
}
fn one() -> usize {
    1
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Shape {
    /// Walls around a floor, with doors in the walls.
    Room {
        layer: usize,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        #[serde(default = "wall")]
        wall: char,
        #[serde(default = "floor")]
        floor: char,
        #[serde(default)]
        doors: Vec<[usize; 2]>,
        #[serde(default = "door")]
        door: char,
    },
    /// An L-shaped path: along x from `from`, then along y to `to`.
    Corridor {
        layer: usize,
        from: [usize; 2],
        to: [usize; 2],
        #[serde(default = "one")]
        width: usize,
        #[serde(default = "floor")]
        floor: char,
    },
    /// A column of stairs on every layer from `from_layer` to `to_layer`.
    Stairs {
        x: usize,
        y: usize,
        from_layer: usize,
        to_layer: usize,
        #[serde(default = "stairs")]
        block: char,
    },
    /// A box of one block on one layer, or on `layers = [low, high]`.
    Fill {
        layer: Option<usize>,
        layers: Option<[usize; 2]>,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        block: char,
    },
    /// One block or spawn point.
    Put {
        layer: usize,
        x: usize,
        y: usize,
        block: char,
    },
}

impl Plan {
    pub fn parse(text: &str) -> Result<Plan, String> {
        toml::from_str(text).map_err(|e| e.to_string())
    }

    /// The plan's layers, bottom first, each as rows joined by newlines.
    pub fn compile(&self) -> Result<Vec<String>, String> {
        let mut grids = vec![vec![vec![self.background; self.width]; self.height]; self.layers];
        let mut set = |l: usize, x: usize, y: usize, c: char| -> Result<(), String> {
            let cell = grids
                .get_mut(l)
                .and_then(|g| g.get_mut(y))
                .and_then(|r| r.get_mut(x))
                .ok_or_else(|| {
                    format!(
                        "({x}, {y}) on layer {l} is outside the {}x{}x{} plan",
                        self.width, self.height, self.layers
                    )
                })?;
            *cell = c;
            Ok(())
        };
        let mut boxed = |l: usize, x: usize, y: usize, w: usize, h: usize, c: char| {
            for yy in y..y + h {
                for xx in x..x + w {
                    set(l, xx, yy, c)?;
                }
            }
            Ok::<(), String>(())
        };

        for (i, shape) in self.shape.iter().enumerate() {
            let at = |e: String| format!("shape {i}: {e}");
            match *shape {
                Shape::Room {
                    layer,
                    x,
                    y,
                    w,
                    h,
                    wall,
                    floor,
                    ref doors,
                    door,
                } => {
                    if w < 3 || h < 3 {
                        return Err(at(format!(
                            "a room needs w and h of at least 3, got {w}x{h}"
                        )));
                    }
                    boxed(layer, x, y, w, h, wall).map_err(at)?;
                    boxed(layer, x + 1, y + 1, w - 2, h - 2, floor).map_err(at)?;
                    for &[dx, dy] in doors {
                        let on_wall = (dx == x || dx == x + w - 1) && (y..y + h).contains(&dy)
                            || (dy == y || dy == y + h - 1) && (x..x + w).contains(&dx);
                        if !on_wall {
                            return Err(at(format!("door ({dx}, {dy}) is not on the room's wall")));
                        }
                        boxed(layer, dx, dy, 1, 1, door).map_err(at)?;
                    }
                }
                Shape::Corridor {
                    layer,
                    from: [fx, fy],
                    to: [tx, ty],
                    width,
                    floor,
                } => {
                    let (x0, x1) = (fx.min(tx), fx.max(tx));
                    let (y0, y1) = (fy.min(ty), fy.max(ty));
                    boxed(layer, x0, fy, x1 - x0 + width, width, floor).map_err(at)?;
                    boxed(layer, tx, y0, width, y1 - y0 + width, floor).map_err(at)?;
                }
                Shape::Stairs {
                    x,
                    y,
                    from_layer,
                    to_layer,
                    block,
                } => {
                    for l in from_layer.min(to_layer)..=from_layer.max(to_layer) {
                        boxed(l, x, y, 1, 1, block).map_err(at)?;
                    }
                }
                Shape::Fill {
                    layer,
                    layers,
                    x,
                    y,
                    w,
                    h,
                    block,
                } => {
                    let [lo, hi] = match (layer, layers) {
                        (Some(l), None) => [l, l],
                        (None, Some([a, b])) => [a.min(b), a.max(b)],
                        _ => return Err(at("a fill needs either layer or layers".to_string())),
                    };
                    for l in lo..=hi {
                        boxed(l, x, y, w, h, block).map_err(at)?;
                    }
                }
                Shape::Put { layer, x, y, block } => {
                    boxed(layer, x, y, 1, 1, block).map_err(at)?;
                }
            }
        }
        Ok(grids
            .into_iter()
            .map(|g| {
                g.into_iter()
                    .map(|row| row.into_iter().collect::<String>())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .collect())
    }
}

/// Layers as the text of a `shape_layers.txt`.
pub fn layers_file(layers: &[String]) -> String {
    let mut text = layers.join("\n~~~~~\n");
    text.push('\n');
    text
}

/// Compile a `.lrstructure` folder's `plan.toml` into its `shape_layers.txt` and merge the
/// plan's legend into `data.json`. Files are only written when their content would
/// change, so building twice changes nothing. Returns whether anything was written;
/// folders without a plan are left alone.
pub fn build_folder(dir: &std::path::Path) -> Result<bool, String> {
    let read = |name: &str| std::fs::read_to_string(dir.join(name));
    let Ok(text) = read("plan.toml") else {
        return Ok(false);
    };
    let folder = dir.display();
    let plan = Plan::parse(&text).map_err(|e| format!("{folder}/plan.toml: {e}"))?;
    let layers = plan
        .compile()
        .map_err(|e| format!("{folder}/plan.toml: {e}"))?;
    let mut wrote = false;

    let current: Vec<String> = read("shape_layers.txt")
        .unwrap_or_default()
        .split("~~~~~")
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    if current != layers {
        std::fs::write(dir.join("shape_layers.txt"), layers_file(&layers))
            .map_err(|e| format!("{folder}/shape_layers.txt: {e}"))?;
        wrote = true;
    }

    let mut data: serde_json::Value = match read("data.json") {
        Ok(text) => serde_json::from_str(&text).map_err(|e| format!("{folder}/data.json: {e}"))?,
        Err(_) => serde_json::json!({
            "blocks": {},
            "gen_biomes": "QUEST_ONLY",
            "gen_chance": 0.0,
            "y_layer_gen_range": [0, 0],
        }),
    };
    let blocks = data["blocks"]
        .as_object_mut()
        .ok_or_else(|| format!("{folder}/data.json: blocks must be a map"))?;
    let mut legend_changed = false;
    for (symbol, block) in &plan.legend {
        if blocks.get(symbol).and_then(|b| b.as_str()) != Some(block.as_str()) {
            blocks.insert(symbol.clone(), serde_json::Value::String(block.clone()));
            legend_changed = true;
        }
    }
    if legend_changed || !dir.join("data.json").exists() {
        let mut text = serde_json::to_string_pretty(&data).expect("json");
        text.push('\n');
        std::fs::write(dir.join("data.json"), text)
            .map_err(|e| format!("{folder}/data.json: {e}"))?;
        wrote = true;
    }
    Ok(wrote)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO_ROOMS: &str = r##"
layers = 2
width = 13
height = 5

[legend]
"#" = "rock"
" " = "air"
"D" = "door"
">" = "stairs"
"." = "existing_worldgen"

[[shape]]
kind = "room"
layer = 0
x = 0
y = 0
w = 5
h = 5
doors = [[4, 2]]

[[shape]]
kind = "room"
layer = 0
x = 8
y = 0
w = 5
h = 5
doors = [[8, 2]]

[[shape]]
kind = "corridor"
layer = 0
from = [5, 2]
to = [7, 2]

[[shape]]
kind = "stairs"
x = 2
y = 2
from_layer = 0
to_layer = 1
"##;

    #[test]
    fn two_rooms_a_corridor_and_stairs_compile_to_layers() {
        let layers = Plan::parse(TWO_ROOMS).unwrap().compile().unwrap();
        assert_eq!(
            layers,
            vec![
                "#####...#####\n\
                 #   #...#   #\n\
                 # > D   D   #\n\
                 #   #...#   #\n\
                 #####...#####",
                ".............\n\
                 .............\n\
                 ..>..........\n\
                 .............\n\
                 ............."
            ]
        );
    }

    #[test]
    fn building_writes_the_layers_and_legend_once() {
        let dir = std::env::temp_dir().join(format!(
            "lithicrivers-plan-{}.lrstructure",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("plan.toml"), TWO_ROOMS).unwrap();
        assert_eq!(build_folder(&dir), Ok(true));
        let layers = std::fs::read_to_string(dir.join("shape_layers.txt")).unwrap();
        assert!(layers.starts_with("#####...#####\n"), "{layers}");
        let data = std::fs::read_to_string(dir.join("data.json")).unwrap();
        assert!(data.contains(r#""D": "door""#), "{data}");
        // A second build changes nothing
        assert_eq!(build_folder(&dir), Ok(false));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn mistakes_name_the_shape() {
        let off_grid = Plan::parse(&TWO_ROOMS.replace("x = 8", "x = 10"))
            .unwrap()
            .compile();
        assert!(off_grid.unwrap_err().starts_with("shape 1:"));
        let bad_door = Plan::parse(&TWO_ROOMS.replace("[[4, 2]]", "[[2, 2]]"))
            .unwrap()
            .compile();
        assert!(bad_door.unwrap_err().contains("not on the room's wall"));
        assert!(Plan::parse("layers = 1\nwidth = 1\nheight = 1\ncolour = 3").is_err());
    }
}

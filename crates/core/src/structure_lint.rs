//! Checks a structure before it reaches a world: equal rectangular layers, every
//! character in the legend, and stairs that line up across layers.

use crate::structure::StructureDefinition;

/// The structure's layers as grids of characters.
pub fn grids(def: &StructureDefinition) -> Vec<Vec<Vec<char>>> {
    def.layers
        .iter()
        .map(|layer| layer.lines().map(|row| row.chars().collect()).collect())
        .collect()
}

/// Every problem found, one line each naming the layer, position and character.
pub fn lint(def: &StructureDefinition) -> Vec<String> {
    let name = &def.name;
    let grids = grids(def);
    let mut problems = Vec::new();
    let Some(first) = grids.first() else {
        return vec![format!("{name}: no layers")];
    };
    let (width, height) = (first.first().map_or(0, Vec::len), first.len());

    for (li, grid) in grids.iter().enumerate() {
        if grid.len() != height {
            problems.push(format!(
                "{name}: layer {li} has {} rows, layer 0 has {height}",
                grid.len()
            ));
        }
        for (y, row) in grid.iter().enumerate() {
            if row.len() != width {
                problems.push(format!(
                    "{name}: layer {li} row {y} is {} wide, layer 0 is {width}",
                    row.len()
                ));
            }
            for (x, ch) in row.iter().enumerate() {
                if def.get_tile_for_symbol(&ch.to_string()).is_none() {
                    problems.push(format!(
                        "{name}: layer {li} ({x}, {y}): '{ch}' is not in the legend"
                    ));
                }
            }
        }
    }

    // A stairs tile leads to stairs on the layer above or below
    let stairs = |li: usize, x: usize, y: usize| {
        grids
            .get(li)
            .and_then(|g| g.get(y))
            .and_then(|r| r.get(x))
            .is_some_and(|&c| {
                def.get_tile_for_symbol(&c.to_string()) == Some(crate::tiles::TileKind::Stairs)
            })
    };
    if grids.len() > 1 {
        for (li, grid) in grids.iter().enumerate() {
            for (y, row) in grid.iter().enumerate() {
                for x in 0..row.len() {
                    if stairs(li, x, y)
                        && !stairs(li + 1, x, y)
                        && !(li > 0 && stairs(li - 1, x, y))
                    {
                        problems.push(format!(
                            "{name}: layer {li} ({x}, {y}): stairs with no stairs above or below"
                        ));
                    }
                }
            }
        }
    }
    problems
}

/// For a structure with a `plan.toml`: problems if its layers or legend aren't what the
/// plan compiles to (someone edited the generated files, or forgot to rebuild).
pub fn plan_problems(folder: &str) -> Vec<String> {
    let Some(text) = crate::structure::embedded_text(&format!("structures/{folder}/plan.toml"))
    else {
        return Vec::new();
    };
    let plan = match crate::structure_plan::Plan::parse(&text) {
        Ok(plan) => plan,
        Err(e) => return vec![format!("{folder}: plan.toml: {e}")],
    };
    let layers = match plan.compile() {
        Ok(layers) => layers,
        Err(e) => return vec![format!("{folder}: plan.toml: {e}")],
    };
    let def = StructureDefinition::load_from_embedded(folder);
    let mut problems = Vec::new();
    if layers != def.layers {
        problems.push(format!(
            "{folder}: shape_layers.txt differs from plan.toml; run `just util-structure-build` and edit the plan, not the layers"
        ));
    }
    for (symbol, block) in &plan.legend {
        if def.blocks.get(symbol) != Some(block) {
            problems.push(format!(
                "{folder}: data.json lacks the plan's legend entry '{symbol}' = '{block}'; run `just util-structure-build`"
            ));
        }
    }
    problems
}

/// A structure's folder name from "small_temple" or "small_temple.lrstructure".
pub fn find_structure(name: &str) -> Result<String, String> {
    let folder = if name.ends_with(".lrstructure") {
        name.to_string()
    } else {
        format!("{name}.lrstructure")
    };
    let names = crate::structure::embedded_structure_names();
    if names.contains(&folder) {
        Ok(folder)
    } else {
        let list: Vec<&str> = names
            .iter()
            .map(|n| n.trim_end_matches(".lrstructure"))
            .collect();
        Err(format!(
            "unknown structure '{name}'; structures: {}",
            list.join(", ")
        ))
    }
}

/// A structure as text: its size and legend, each layer from the bottom up, and what the
/// lint finds. What `just util-structure-preview` prints.
pub fn preview(name: &str) -> Result<String, String> {
    let folder = find_structure(name)?;
    let def = StructureDefinition::load_from_embedded(&folder);
    let grids = grids(&def);
    let (width, height) = (
        grids.first().and_then(|g| g.first()).map_or(0, Vec::len),
        grids.first().map_or(0, Vec::len),
    );
    let source =
        if crate::structure::embedded_text(&format!("structures/{folder}/plan.toml")).is_some() {
            "plan.toml"
        } else {
            "hand-written shape_layers.txt"
        };
    let mut out = format!(
        "{}: {width} wide, {height} tall, {} layers (from {source})\n",
        def.name,
        grids.len()
    );
    out.push_str("Legend:\n");
    let mut legend: Vec<_> = def.blocks.iter().collect();
    legend.sort();
    for (symbol, block) in legend {
        out.push_str(&format!("  '{symbol}' {block}\n"));
    }
    out.push_str(
        "Layer 0 is the bottom. A buried structure puts its top layer at the placement z.\n",
    );
    for (li, layer) in def.layers.iter().enumerate() {
        out.push_str(&format!("--- layer {li} ---\n{layer}\n"));
    }
    let mut problems = lint(&def);
    problems.extend(plan_problems(&folder));
    if problems.is_empty() {
        out.push_str("No problems found.\n");
    } else {
        out.push_str("Problems:\n");
        for p in problems {
            out.push_str(&format!("  {p}\n"));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::{embedded_structure_names, StructureDefinition};

    #[test]
    fn every_structure_passes_the_lint() {
        let names = embedded_structure_names();
        assert!(names.len() >= 6, "{names:?}");
        let problems: Vec<String> = names
            .iter()
            .flat_map(|n| {
                let mut found = lint(&StructureDefinition::load_from_embedded(n));
                found.extend(plan_problems(n));
                found
            })
            .collect();
        assert!(problems.is_empty(), "\n{}", problems.join("\n"));
    }

    #[test]
    fn small_temple_is_built_from_its_plan() {
        assert!(
            crate::structure::embedded_text("structures/small_temple.lrstructure/plan.toml")
                .is_some()
        );
        assert!(plan_problems("small_temple.lrstructure").is_empty());
    }

    #[test]
    fn small_temple_previews_as_text() {
        let text = preview("small_temple").unwrap();
        assert_eq!(
            text,
            "small_temple: 5 wide, 5 tall, 3 layers (from plan.toml)
Legend:
  ' ' air
  ',' existing_worldgen
  'T' treasure_common
  'X' iron_scrap
Layer 0 is the bottom. A buried structure puts its top layer at the placement z.
--- layer 0 ---
XXXXX
X   X
X T X
X   X
XXXXX
--- layer 1 ---
,,,,,
,XXX,
,XXX,
,XXX,
,,,,,
--- layer 2 ---
,,,,,
,,,,,
,,T,,
,,,,,
,,,,,
No problems found.
"
        );
        let unknown = preview("castle").unwrap_err();
        assert!(unknown.contains("structures: giant_corpse"), "{unknown}");
    }

    #[test]
    fn a_typo_names_the_structure_layer_position_and_character() {
        let mut def = StructureDefinition::load_from_embedded("small_temple.lrstructure");
        let mut rows: Vec<String> = def.layers[0].lines().map(str::to_string).collect();
        rows[0].replace_range(0..1, "@");
        def.layers[0] = rows.join("\n");
        let problems = lint(&def);
        assert!(
            problems
                .iter()
                .any(|p| p.contains("layer 0 (0, 0): '@' is not in the legend")),
            "{problems:?}"
        );
    }
}

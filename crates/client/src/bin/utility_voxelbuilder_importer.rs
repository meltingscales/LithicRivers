/// Voxel Builder Importer for LithicRivers
/// Converts Voxel Builder JSON exports to .lrstructure format

/// Convert hex color to RGB values
fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        (r, g, b)
    } else {
        (0, 0, 0)
    }
}

// No additional imports needed for basic parsing
/// Create mapping from hex colors to LithicRivers block characters
fn create_hex_color_block_mapping(hex_color: &str) -> char {
    let (r, g, b) = hex_to_rgb(hex_color);

    //TODO treasure_rare
    //TODO treasure_quest_1

    match (r, g, b) {
        // Map specific colors to blocks based on RGB values
        (145, 19, 245) => 'E',  // purple (#9113F5) -> Enemy spawns
        (144, 160, 179) => '#', // gray-blue (#90A0B3) -> stone
        (255, 255, 0) => 't',   // yellow (#FFFF00) -> treasure_common
        (131, 50, 0) => 's',    // brown (#833200) -> scrap_common
        (255, 155, 94) => 'S',  // lighter brown (#FF9B5E) -> scrap_rare
        (0, 255, 255) => '!',   // cyan (#00FFFF) -> special_player_spawn
        (0, 255, 0) => 'D',     // green (#00FF00) -> door
        (255, 128, 0) => '>',   // orange (#FF8000) -> stairs
        (0, 0, 0) => ' ',       // black (#000000) -> air
        _ => panic!("{}", format!("Unknown color {}", hex_color)), // Default will panic
    }
}

fn gen_data_json(height: usize) -> serde_json::Value {
    serde_json::json!({
        "blocks": {
            ".": "existing_worldgen",
            " ": "air",
            "#": "rock",
            "E": "enemy_spawn",
            "s": "scrap_common",
            "S": "scrap_rare",
            "t": "treasure_common",
            "T": "treasure_rare",
            "1": "treasure_quest_1",
            "!": "special_player_spawn",
            "D": "door",
            ">": "stairs",
        },
        "gen_biomes": "QUEST_ONLY",
        "gen_chance": 0.0,
        "y_layer_gen_range": [0, height - 1]
    })
}

/// Voxel Builder Importer v1.0
/// Converts Voxel Builder JSON exports to LithicRivers .lrstructure format
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧊 LithicRivers Voxel Builder Importer v1.0");
    println!("=============================================");

    // Define input -> output mappings
    let input_output_map: Vec<(&str, &str)> = vec![
        (
            // 1st quest, escape the bunker
            "crates/client/assets/voxelbuilder/sapiencorp-bunker.voxelbuilder.json",
            "crates/client/assets/structures/sapiencorp-bunker.lrstructure",
        ),
        (
            // 2nd quest, get your arm back
            "crates/client/assets/voxelbuilder/sapiencorp-factory.voxelbuilder.json",
            "crates/client/assets/structures/sapiencorp-factory.lrstructure",
        ),
    ];

    if input_output_map.is_empty() {
        println!("📋 No Voxel Builder files configured for import.");
        println!("   Add JSON files to the input_output_map to process them.");
        println!();
        println!("💡 Workflow:");
        println!(
            "   1. Create voxel models in Voxel Builder (https://nimadez.github.io/voxel-builder/)"
        );
        println!("   2. Export as JSON format");
        println!("   3. Add to input_output_map above");
        println!("   4. Run this importer");
        return Ok(());
    }

    println!("Processing {} structure(s):", input_output_map.len());
    println!();

    for (input_path, output_dir) in input_output_map {
        println!("📂 Processing: {}", input_path);
        println!("   Output: {}", output_dir);

        // Create output directory
        std::fs::create_dir_all(&output_dir)?;

        // Parse the Voxel Builder JSON file
        println!("   🔍 Parsing Voxel Builder JSON...");
        parse_voxelbuilder_json_to_structure(input_path, &output_dir)?;

        println!("   ✓ Structure imported successfully!");
        println!("     - {}/data.json", output_dir);
        println!("     - {}/shape_layers.txt", output_dir);
        println!();
    }

    println!("🎉 All structures processed successfully!");
    Ok(())
}

/// Parse a Voxel Builder JSON file and convert it to LithicRivers structure format
fn parse_voxelbuilder_json_to_structure(
    input_path: &str,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read the JSON file
    let json_content = std::fs::read_to_string(input_path)?;
    let voxel_data: serde_json::Value = serde_json::from_str(&json_content)?;

    println!("   📄 Loaded Voxel Builder JSON");

    // Extract voxel data from the "data.voxels" string
    // Format: "x,y,z,color,material;x,y,z,color,material;..."
    let voxels_string = voxel_data["data"]["voxels"]
        .as_str()
        .ok_or("No voxels data found")?;

    println!("   🔄 Parsing voxel data...");
    let voxels = parse_voxel_string(voxels_string)?;

    println!("   📊 Found {} voxels", voxels.len());

    // Convert voxels to LithicRivers structure format
    let model = VoxelBuilderModel { voxels };

    // Generate the structure files
    create_structure_files(&model, output_dir)?;

    Ok(())
}

#[derive(Debug)]
struct VoxelBuilderVoxel {
    x: i64,
    y: i64,
    z: i64,
    color: String, // Hex color like "9113F5"
}

#[derive(Debug)]
struct VoxelBuilderModel {
    voxels: Vec<VoxelBuilderVoxel>,
}

/// Parse the voxel string format: "x,y,z,color,material;x,y,z,color,material;..."
fn parse_voxel_string(
    voxels_string: &str,
) -> Result<Vec<VoxelBuilderVoxel>, Box<dyn std::error::Error>> {
    let mut voxels = Vec::new();

    // Split by semicolon, filter out empty entries
    for voxel_entry in voxels_string.split(';').filter(|s| !s.trim().is_empty()) {
        let parts: Vec<&str> = voxel_entry.split(',').collect();
        if parts.len() >= 5 {
            let x = parts[0].parse::<i64>()?;
            let y = parts[1].parse::<i64>()?;
            let z = parts[2].parse::<i64>()?;
            let color = parts[3].to_string();

            voxels.push(VoxelBuilderVoxel { x, y, z, color });
        }
    }

    Ok(voxels)
}

/// Convert voxels to 2D layer structure and create the structure files
fn create_structure_files(
    model: &VoxelBuilderModel,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Find the bounds of the model
    let mut min_x = i64::MAX;
    let mut max_x = i64::MIN;
    let mut min_y = i64::MAX;
    let mut max_y = i64::MIN;
    let mut min_z = i64::MAX;
    let mut max_z = i64::MIN;

    for voxel in &model.voxels {
        min_x = min_x.min(voxel.x);
        max_x = max_x.max(voxel.x);
        min_y = min_y.min(voxel.y);
        max_y = max_y.max(voxel.y);
        min_z = min_z.min(voxel.z);
        max_z = max_z.max(voxel.z);
    }

    let width = (max_x - min_x + 1) as usize;
    let depth = (max_z - min_z + 1) as usize;
    let height = (max_y - min_y + 1) as usize;

    println!(
        "   📐 Model dimensions: {}x{}x{} (WxDxH)",
        width, depth, height
    );

    // Create layers (Y is height/layers in Voxel Builder)
    let mut layers = Vec::new();

    for y in min_y..=max_y {
        let mut layer_grid = vec![vec!['.'; width]; depth];

        // Fill in voxels for this Y layer
        for voxel in &model.voxels {
            if voxel.y == y {
                let x_idx = (voxel.x - min_x) as usize;
                let z_idx = (voxel.z - min_z) as usize;
                let block_char = create_hex_color_block_mapping(&voxel.color);
                layer_grid[z_idx][x_idx] = block_char;
            }
        }

        // Convert grid to string
        let layer_string: String = layer_grid
            .iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");

        layers.push(layer_string);
    }

    // Create the structure files
    let shape_layers = layers.join("\n~~~~~\n");
    let shape_path = format!("{}/shape_layers.txt", output_dir);
    std::fs::write(&shape_path, shape_layers)?;

    // Create the JSON metadata
    let data = serde_json::json!(gen_data_json(height));

    let data_path = format!("{}/data.json", output_dir);
    std::fs::write(&data_path, serde_json::to_string_pretty(&data)?)?;

    println!("   💾 Generated structure with {} layers", layers.len());

    Ok(())
}

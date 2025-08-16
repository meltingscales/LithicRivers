use lithicrivers_core::palette::PaletteKey;
use lithicrivers_core::resources::fluids::FluidType;
use lithicrivers_core::tiles::TileKind;
use ratatui::style::Color;

pub fn color_for_key(key: PaletteKey) -> Color {
    match key {
        // Tiles
        PaletteKey::Rock => Color::Rgb(102, 102, 115), // 0.4, 0.4, 0.45 * 255
        PaletteKey::Dirt => Color::Rgb(178, 178, 178), // 0.7, 0.7, 0.7 * 255
        PaletteKey::Grass => Color::Rgb(153, 204, 153), // 0.6, 0.8, 0.6 * 255
        PaletteKey::Tree => Color::Rgb(153, 204, 153), // 0.6, 0.8, 0.6 * 255
        PaletteKey::Air => Color::Rgb(178, 178, 178),  // 0.7, 0.7, 0.7 * 255
        PaletteKey::BoneBlock => Color::Rgb(178, 178, 178),
        PaletteKey::IronScrap => Color::Rgb(178, 178, 178),
        PaletteKey::Door => Color::Rgb(178, 178, 178),
        PaletteKey::Bedrock => Color::Rgb(178, 178, 178),
        PaletteKey::ScrapElectronics => Color::Rgb(178, 178, 178),
        PaletteKey::PlasteelScrap => Color::Rgb(178, 178, 178),
        PaletteKey::Treasure => Color::Rgb(178, 178, 178),
        // Fluids
        PaletteKey::Water => Color::Rgb(76, 127, 255), // 0.3, 0.5, 1.0 * 255
        PaletteKey::Oil => Color::Rgb(153, 153, 153),  // 0.6, 0.6, 0.6 * 255
        PaletteKey::Blood => Color::Rgb(153, 25, 25),  // 0.6, 0.1, 0.1 * 255
        PaletteKey::Acid => Color::Rgb(153, 153, 153),
        PaletteKey::Lava => Color::Rgb(153, 153, 153),
    }
}

pub fn color_for_tile(kind: TileKind) -> Color {
    color_for_key(kind.palette_key())
}

pub fn color_for_fluid(ft: FluidType) -> Color {
    color_for_key(ft.palette_key())
}

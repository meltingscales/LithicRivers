use bevy::prelude::Color;
use lithicrivers_core::palette::PaletteKey;
use lithicrivers_core::tiles::TileKind;
use lithicrivers_core::resources::fluids::FluidType;

pub fn color_for_key(key: PaletteKey) -> Color {
    match key {
        // Tiles
        PaletteKey::Rock => Color::rgb(0.4, 0.4, 0.45),
        PaletteKey::Dirt => Color::rgb(0.7, 0.7, 0.7),
        PaletteKey::Grass => Color::rgb(0.6, 0.8, 0.6),
        PaletteKey::Tree => Color::rgb(0.6, 0.8, 0.6),
        PaletteKey::Air => Color::rgb(0.7, 0.7, 0.7),
        PaletteKey::BoneBlock => Color::rgb(0.7, 0.7, 0.7),
        PaletteKey::IronScrap => Color::rgb(0.7, 0.7, 0.7),
        PaletteKey::Door => Color::rgb(0.7, 0.7, 0.7),
        PaletteKey::Bedrock => Color::rgb(0.7, 0.7, 0.7),
        PaletteKey::ScrapElectronics => Color::rgb(0.7, 0.7, 0.7),
        PaletteKey::PlasteelScrap => Color::rgb(0.7, 0.7, 0.7),
        PaletteKey::Treasure => Color::rgb(0.7, 0.7, 0.7),
        // Fluids
        PaletteKey::Water => Color::rgb(0.3, 0.5, 1.0),
    }
}

pub fn color_for_tile(kind: TileKind) -> Color {
    color_for_key(kind.palette_key())
}

pub fn color_for_fluid(ft: FluidType) -> Color {
    color_for_key(ft.palette_key())
}

use bevy::prelude::*;
use bevy::sprite::{TextureAtlas, TextureAtlasSprite, SpriteSheetBundle};
use bevy::render::texture::ImageSampler;
use ab_glyph::{FontArc, PxScale};
use ab_glyph::Font as AbGlyphFont;

// Simple demo app that builds the ASCII atlas (32..=126) and renders all cells
// in a separate window.

#[derive(Resource, Debug, Default)]
struct AsciiAtlasDemo {
    atlas: Handle<TextureAtlas>,
    built: bool,
    cols: u32,
    rows: u32,
    tile_px: u32,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.12, 0.12, 0.12)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "AsciiAtlas Demo".to_string(),
                resolution: (800., 600.).into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .insert_resource(AsciiAtlasDemo::default())
        .add_systems(Startup, (setup_camera, build_ascii_atlas))
        .add_systems(Update, spawn_grid_once)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn build_ascii_atlas(
    mut atlas: ResMut<AsciiAtlasDemo>,
    mut images: ResMut<Assets<Image>>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    if atlas.built { return; }

    // Load font bytes directly; try common repo-relative locations
    let candidate_paths = [
        "crates/client/assets/fonts/monospace.ttf", // run from repo root
        "assets/fonts/monospace.ttf",               // run from client crate dir
    ];
    let mut bytes_opt: Option<Vec<u8>> = None;
    for p in candidate_paths.iter() {
        match std::fs::read(p) {
            Ok(b) => { info!("demo: loaded font from {}", p); bytes_opt = Some(b); break; }
            Err(_) => { trace!("demo: font not at {}", p); }
        }
    }
    let Some(bytes) = bytes_opt else {
        warn!("demo: could not find monospace.ttf in expected paths; skipping atlas build");
        return;
    };

    let Ok(font) = FontArc::try_from_vec(bytes) else {
        warn!("demo: failed to parse monospace.ttf; skipping atlas build");
        return;
    };

    let tile = 32u32; // demo tile size
    let cols = 16u32;
    let rows = 6u32; // 16 * 6 = 96 slots (one unused for 95 printable ASCII)
    let cell_w = tile;
    let cell_h = tile;
    let img_w = cols * cell_w;
    let img_h = rows * cell_h;
    let mut pixels = vec![0u8; (img_w * img_h * 4) as usize];

    // Choose scale to fit height with some margin
    let scale = PxScale { x: (tile as f32) * 0.9, y: (tile as f32) * 0.9 };

    let mut index = 0usize;
    for code in 32u8..=126u8 {
        let ch = code as char;
        let col = (index as u32) % cols;
        let row = (index as u32) / cols;
        let origin_x = (col * cell_w) as i32;
        let origin_y = (row * cell_h) as i32;

        let glyph_id = font.glyph_id(ch);
        let glyph = glyph_id.with_scale(scale);
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|x, y, v| {
                let gx = origin_x + x as i32 + ((cell_w as i32) / 2 - ((bounds.max.x - bounds.min.x) as i32)/2);
                let gy = origin_y + y as i32 + ((cell_h as i32) / 2 - ((bounds.max.y - bounds.min.y) as i32)/2);
                if gx < 0 || gy < 0 { return; }
                let gx = gx as u32; let gy = gy as u32;
                if gx >= img_w || gy >= img_h { return; }
                let idx = ((gy * img_w + gx) * 4) as usize;
                let a = (v.clamp(0.0, 1.0) * 255.0) as u8;
                pixels[idx + 0] = 255;
                pixels[idx + 1] = 255;
                pixels[idx + 2] = 255;
                pixels[idx + 3] = a;
            });
        }
        index += 1;
    }

    // Upload to Image and set nearest sampling
    let mut image = Image::new_fill(
        bevy::render::render_resource::Extent3d { width: img_w, height: img_h, depth_or_array_layers: 1 },
        bevy::render::render_resource::TextureDimension::D2,
        &pixels,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
    );
    image.sampler = ImageSampler::nearest();
    let texture = images.add(image);

    // Create texture atlas from grid
    let atlas_asset = TextureAtlas::from_grid(texture.clone(), Vec2::new(cell_w as f32, cell_h as f32), cols as usize, rows as usize, None, None);
    let atlas_handle = atlases.add(atlas_asset);

    atlas.atlas = atlas_handle;
    atlas.cols = cols;
    atlas.rows = rows;
    atlas.tile_px = tile;
    atlas.built = true;
    info!("demo: ASCII atlas built: {}x{}, cells={}x{}", img_w, img_h, cols, rows);
}

#[derive(Component)]
struct AtlasCell(usize);

fn spawn_grid_once(
    mut commands: Commands,
    atlas: Res<AsciiAtlasDemo>,
    q: Query<Entity, With<AtlasCell>>,
) {
    if !atlas.built { return; }
    if !q.is_empty() { return; }

    // Resize window roughly to fit content
    let width = (atlas.cols as f32) * (atlas.tile_px as f32) + 32.0;
    let height = (atlas.rows as f32) * (atlas.tile_px as f32) + 32.0;
    // Adjust primary window size
    commands.spawn((
        // Marker entity for grouping (optional in this simple demo)
        Name::new("AtlasRoot"),
    ));

    let sx = atlas.tile_px as f32;
    let sy = atlas.tile_px as f32;
    let half_cols = (atlas.cols as i32) / 2;
    let half_rows = (atlas.rows as i32) / 2;

    for idx in 0..(atlas.cols * atlas.rows) {
        let idx_usize = idx as usize;
        let col = (idx as i32) % (atlas.cols as i32);
        let row = (idx as i32) / (atlas.cols as i32);
        let tx = (col - half_cols) as f32 * sx;
        let ty = (row - half_rows) as f32 * -sy;
        commands.spawn((
            SpriteSheetBundle {
                texture_atlas: atlas.atlas.clone(),
                sprite: TextureAtlasSprite {
                    index: idx_usize,
                    color: Color::WHITE,
                    custom_size: Some(Vec2::splat(atlas.tile_px as f32)),
                    ..Default::default()
                },
                transform: Transform::from_translation(Vec3::new(tx, ty, 0.0)),
                ..Default::default()
            },
            AtlasCell(idx_usize),
        ));
    }

    // Also set window size if possible (requires primary window access via a system; omitted for brevity)
    info!("demo: spawned {} atlas cells", atlas.cols * atlas.rows);
}

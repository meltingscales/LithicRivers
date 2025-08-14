use bevy::prelude::*;
use bevy::sprite::{TextureAtlas, TextureAtlasSprite, SpriteSheetBundle};
use bevy::input::mouse::{MouseMotion};
use bevy::window::CursorGrabMode;
use lithicrivers_core::Game;
use lithicrivers_core::tiles::TileKind;
use lithicrivers_core::resources::world::CHUNK_SIZE;
use std::collections::{HashMap, HashSet, VecDeque};
use bevy::render::texture::ImageSampler;
use bevy::utils::tracing::info_span;
use bevy::render::render_resource::Extent3d;
mod sprite_loader;
use crate::sprite_loader::{SpriteLoader, SpriteData};
use ab_glyph::{FontArc, PxScale, point};
use ab_glyph::Font as AbGlyphFont;
use bevy::text::Font as BevyFont;
mod palette;
use crate::palette::color_for_tile;
use crate::palette::color_for_fluid;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn div_floor_basics() {
        assert_eq!(div_floor(0, CHUNK_SIZE), 0);
        assert_eq!(div_floor(CHUNK_SIZE - 1, CHUNK_SIZE), 0);
        assert_eq!(div_floor(CHUNK_SIZE, CHUNK_SIZE), 1);
        assert_eq!(div_floor(-1, CHUNK_SIZE), -1);
        assert_eq!(div_floor(-CHUNK_SIZE, CHUNK_SIZE), -1);
        assert_eq!(div_floor(-CHUNK_SIZE-1, CHUNK_SIZE), -2);
    }

    #[test]
    fn world_to_chunk_edges() {
        let c = world_to_chunk_2d(0, 0, CHUNK_SIZE);
        assert_eq!(c.cx, 0); assert_eq!(c.cy, 0);
        let c = world_to_chunk_2d(CHUNK_SIZE-1, CHUNK_SIZE-1, CHUNK_SIZE);
        assert_eq!(c.cx, 0); assert_eq!(c.cy, 0);
        let c = world_to_chunk_2d(CHUNK_SIZE, 0, CHUNK_SIZE);
        assert_eq!(c.cx, 1); assert_eq!(c.cy, 0);
        let c = world_to_chunk_2d(-1, -1, CHUNK_SIZE);
        assert_eq!(c.cx, -1); assert_eq!(c.cy, -1);
        let c = world_to_chunk_2d(-CHUNK_SIZE, -CHUNK_SIZE, CHUNK_SIZE);
        assert_eq!(c.cx, -1); assert_eq!(c.cy, -1);
        let c = world_to_chunk_2d(-CHUNK_SIZE-1, -CHUNK_SIZE-1, CHUNK_SIZE);
        assert_eq!(c.cx, -2); assert_eq!(c.cy, -2);
    }

    #[test]
    fn hash64_is_deterministic_and_changes() {
        let s = 123u64;
        let a = hash64(s, 0, 0);
        let b = hash64(s, 0, 0);
        assert_eq!(a, b);
        let c = hash64(s, 1, 0);
        let d = hash64(s, 0, 1);
        assert_ne!(a, c);
        assert_ne!(a, d);
    }

    #[test]
    fn generate_chunk_is_deterministic() {
        let cc = ChunkCoord2D { cx: 5, cy: -2 };
        let a = generate_chunk_2d(9999, cc, CHUNK_SIZE, CHUNK_SIZE);
        let b = generate_chunk_2d(9999, cc, CHUNK_SIZE, CHUNK_SIZE);
        assert_eq!(a.w, b.w);
        assert_eq!(a.tiles.len(), b.tiles.len());
        for (i, (ta, tb)) in a.tiles.iter().zip(b.tiles.iter()).enumerate() {
            assert_eq!(ta.kind as u8, tb.kind as u8, "tile mismatch at {}", i);
        }
    }

    #[test]
    fn tile_color_mapping_matches() {
        assert_eq!(tile_color(TileKind::Rock), Color::rgb(0.4, 0.4, 0.45));
        assert_eq!(tile_color(TileKind::Dirt), Color::rgb(0.7, 0.7, 0.7));
        assert_eq!(tile_color(TileKind::Grass), Color::rgb(0.6, 0.8, 0.6));
        assert_eq!(tile_color(TileKind::Tree), Color::rgb(0.6, 0.8, 0.6));
        assert_eq!(tile_color(TileKind::Air), Color::rgb(0.7, 0.7, 0.7));
    }

    #[test]
    fn demo_tile_at_is_stable() {
        let k1 = demo_tile_at(3, -7);
        let k2 = demo_tile_at(3, -7);
        assert_eq!(k1 as u8, k2 as u8);
    }
}
#[derive(Resource)]
struct CoreGame(pub Game);

#[derive(Resource, Deref, DerefMut)]
struct FixedTickTimer(Timer);

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Copy, Default)]
enum ViewMode {
    #[default]
    TwoD,
    ThreeD,
}

// Map shared TileKind to a Bevy Color (delegates to client palette)
fn tile_color(kind: TileKind) -> Color { color_for_tile(kind) }

// Very small deterministic demo for 3D using the same hashing approach
#[cfg(test)]
fn demo_tile_at(x: i32, z: i32) -> TileKind {
    // These constants mirror the 2D generate logic but keep it simple.
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    let seed = 12345u64; // demo seed; in future, pull WorldSeed
    let cc = world_to_chunk_2d(x, z, CHUNK_SIZE);
    let local_x = (x.rem_euclid(CHUNK_SIZE)) as i32;
    let local_z = (z.rem_euclid(CHUNK_SIZE)) as i32;
    // Seed RNG per chunk like 2D, then advance in a stable way for local cell
    let mut rng = ChaCha20Rng::seed_from_u64(hash64(seed, cc.cx as i64, cc.cy as i64));
    // Advance RNG index by a stable offset per local cell to keep distribution consistent
    let steps = (local_z * CHUNK_SIZE + local_x) as usize;
    for _ in 0..steps { let _: f32 = rng.gen(); }
    let r: f32 = rng.gen();
    if r < 0.10 { TileKind::Rock }
    else if r < 0.50 { TileKind::Dirt }
    else { TileKind::Grass }
}

#[derive(Component)]
struct View2D;

#[derive(Component)]
struct View3D;

#[derive(Component)]
struct Ground3D;

#[derive(Component)]
struct PlayerMarker;

// 2D ASCII view config and resources
#[derive(Resource, Debug, Clone, Copy)]
struct View2DConfig {
    tile_px: f32,
    _cols: i32,
    _rows: i32,
    chunk_size: i32,
    view_chunk_radius: i32,
}

// 3D view config (chunk-based limits)
#[derive(Resource, Debug, Clone, Copy)]
struct View3DConfig {
    chunk_size: i32,
    view_chunk_radius: i32,
}

#[derive(Resource, Debug, Clone, Copy)]
struct WorldSeed(pub u64);

#[derive(Component)]
struct AsciiRoot; // Parent for all ASCII glyphs so we can clear easily
#[derive(Component)]
struct AsciiCell(usize); // row-major index within the visible grid

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
struct ChunkCoord2D { cx: i32, cy: i32 }

#[derive(Clone)]
struct TileCell { kind: TileKind }

#[derive(Clone)]
struct ChunkData2D {
    w: i32,
    tiles: Vec<TileCell>,
}

#[derive(Resource, Default)]
struct LoadedChunks2D {
    map: HashMap<ChunkCoord2D, ChunkData2D>,
    lru: VecDeque<ChunkCoord2D>,
    capacity: usize,
}

#[derive(Resource, Default)]
struct VisibleChunks2D(HashSet<ChunkCoord2D>);

#[derive(Resource, Default)]
struct FontHandles {
    primary: Handle<BevyFont>,   // assets/fonts/monospace.ttf
    initiated: bool,
}

// Prebaked ASCII atlas generated from monospace.ttf for fast 2D rendering
#[derive(Resource, Default)]
struct AsciiAtlas {
    atlas: Handle<TextureAtlas>,
    // map ascii char -> atlas index
    map: HashMap<char, usize>,
    built: bool,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
struct AsciiGrid { cols: i32, rows: i32, initialized: bool }

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "LithicRivers (Bevy client)".to_string(),
                resolution: (1280., 720.).into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .insert_resource(CoreGame(Game::new(12345)))
        .insert_resource(WorldSeed(12345))
        .insert_resource(FixedTickTimer(Timer::from_seconds(1.0/30.0, TimerMode::Repeating)))
        .insert_resource(SpriteLoader::new(None))
        .init_resource::<AsciiAtlas>()
        .init_resource::<Last2DPos>()
        .init_resource::<Camera3DRotation>()
        .init_resource::<AsciiGrid>()
        .insert_resource(View2DConfig { tile_px: 16.0, _cols: 80, _rows: 45, chunk_size: lithicrivers_core::resources::world::CHUNK_SIZE, view_chunk_radius: 2 })
        .insert_resource(View3DConfig { chunk_size: CHUNK_SIZE, view_chunk_radius: 2 })
        .insert_resource(LoadedChunks2D { map: HashMap::new(), lru: VecDeque::new(), capacity: 256 })
        .insert_resource(VisibleChunks2D(HashSet::new()))
        .init_resource::<FontHandles>()
        .insert_resource(SpriteLoader::new(None))
        .add_state::<ViewMode>()
        // Build ASCII atlas once at startup
        .add_systems(Startup, build_ascii_atlas)
        // Common input and mode toggling
        .add_systems(Update, (toggle_view_mode, keyboard_input_system))
        // 2D view lifecycle
        .add_systems(OnEnter(ViewMode::TwoD), setup_2d)
        .add_systems(OnExit(ViewMode::TwoD), teardown_2d)
        // 2D update systems
        .add_systems(Update, (
            update_visible_chunks_2d,
            ensure_chunks_loaded_2d,
            render_ascii_2d,
        ).run_if(in_state(ViewMode::TwoD)))
        // 3D lifecycle
        .add_systems(OnEnter(ViewMode::ThreeD), setup_3d)
        .add_systems(OnExit(ViewMode::ThreeD), teardown_3d)
        .add_systems(Update, update_player_marker_3d.run_if(in_state(ViewMode::ThreeD)))
        .add_systems(Update, camera_3d_mouse_rotation.run_if(in_state(ViewMode::ThreeD)))
        .add_systems(Update, update_camera_3d_follow_player.run_if(in_state(ViewMode::ThreeD)))
        .add_systems(Update, update_compass_ui.run_if(in_state(ViewMode::ThreeD)))
        .add_systems(Update, (
            update_visible_chunks_3d,
            ensure_chunks_loaded_3d,
        ).run_if(in_state(ViewMode::ThreeD)))
        .add_systems(Update, update_ground_3d.run_if(in_state(ViewMode::ThreeD)))
        .run();
}
#[derive(Resource, Debug)]
struct Camera3DRotation {
    yaw: f32,
    pitch: f32,
    mouse_held: bool,
}

impl Default for Camera3DRotation {
    fn default() -> Self {
        Self { yaw: 0.0, pitch: 0.0, mouse_held: false }
    }
}

#[derive(Component, Clone, Copy)]
struct CompassDir(char);
#[derive(Component)]
struct CompassUI;

fn update_compass_ui(
    rot: Res<Camera3DRotation>,
    mut q: Query<(&mut Transform, &CompassDir, &mut Text)>,
) {
    // Place N, E, S, W in a circle, rotated by camera yaw
    let radius = 40.0;
    let base_angles = [0.0, std::f32::consts::FRAC_PI_2, std::f32::consts::PI, 3.0*std::f32::consts::FRAC_PI_2]; // N, E, S, W
    for (mut transform, dir, mut text) in &mut q {
        let i = match dir.0 {
            'N' => 0,
            'E' => 1,
            'S' => 2,
            'W' => 3,
            _ => continue,
        };
        let angle = base_angles[i] - rot.yaw;
        let x = radius * angle.sin();
        let y = -radius * angle.cos(); // y down in UI
        transform.translation = Vec3::new(x, y, 0.0);
        transform.rotation = Quat::IDENTITY;
        text.sections[0].value = dir.0.to_string();
    }
}


fn camera_3d_mouse_rotation(
    mut rot: ResMut<Camera3DRotation>,
    mut mouse_events: EventReader<MouseMotion>,
    mouse_btn: Res<Input<MouseButton>>,
    mut windows: Query<&mut Window>,
) {
    // Only rotate if right mouse is held
    let held = mouse_btn.pressed(MouseButton::Right);
    rot.mouse_held = held;
    if held {
        let mut dx = 0.0;
        let mut dy = 0.0;
        for ev in mouse_events.read() {
            dx += ev.delta.x;
            dy += ev.delta.y;
        }
        rot.yaw -= dx * 0.01;
        rot.pitch -= dy * 0.01;
        rot.pitch = rot.pitch.clamp(-1.5, 1.5);
        // Optionally grab cursor
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor.grab_mode = CursorGrabMode::Locked;
            window.cursor.visible = false;
        }
    } else {
        // Release cursor
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        }
    }
}

fn setup_3d(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _loaded: Res<LoadedChunks2D>,
) {
    // Camera (will be updated to follow player)
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(8.0, 8.0, 16.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        View3D,
    ));
    // Light
    commands.spawn((
        PointLightBundle {
            transform: Transform::from_xyz(4.0, 8.0, 8.0),
            ..Default::default()
        },
        View3D,
    ));
    // Player cube (will be updated to match player position)
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(bevy::prelude::shape::Box::new(0.9, 0.9, 0.9))),
            material: materials.add(Color::rgb(0.9, 0.9, 0.9).into()),
            transform: Transform::from_xyz(0.0, 0.6, 0.0),
            ..Default::default()
        },
        PlayerMarker,
        View3D,
    ));
    // (Ground tiles are now spawned dynamically by update_ground_3d)

    // Compass UI (NESW, top right)
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(120.0),
                height: Val::Px(120.0),
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            background_color: BackgroundColor(Color::NONE),
            ..Default::default()
        },
        CompassUI,
        View3D,
    )).with_children(|parent| {
        // Four separate TextBundles for N, E, S, W
        let font = Handle::default(); // replaced at runtime
        let style = Style {
            position_type: PositionType::Absolute,
            ..Default::default()
        };
        parent.spawn((TextBundle {
            text: Text::from_section("N", TextStyle { font: font.clone(), font_size: 26.0, color: Color::BLACK }),
            style: style.clone(),
            ..Default::default()
        }, CompassDir('N')));
        parent.spawn((TextBundle {
            text: Text::from_section("E", TextStyle { font: font.clone(), font_size: 26.0, color: Color::BLACK }),
            style: style.clone(),
            ..Default::default()
        }, CompassDir('E')));
        parent.spawn((TextBundle {
            text: Text::from_section("S", TextStyle { font: font.clone(), font_size: 26.0, color: Color::BLACK }),
            style: style.clone(),
            ..Default::default()
        }, CompassDir('S')));
        parent.spawn((TextBundle {
            text: Text::from_section("W", TextStyle { font: font, font_size: 26.0, color: Color::BLACK }),
            style: style,
            ..Default::default()
        }, CompassDir('W')));
    });
}

fn teardown_3d(
    mut commands: Commands,
    q: Query<Entity, With<View3D>>,
    q_compass: Query<Entity, With<CompassUI>>,
    mut last3d: ResMut<Last3DPos>,
) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
    for e in &q_compass {
        commands.entity(e).despawn_recursive();
    }
    // Force re-render next time 3D is entered
    *last3d = Last3DPos(i32::MIN, i32::MIN);
}

fn setup_2d(mut commands: Commands, mut last2d: ResMut<Last2DPos>) {
    info!("setup_2d: creating camera, root, and player marker");
    // Camera
    commands.spawn((Camera2dBundle::default(), View2D));
    // Root for ASCII glyphs (spawned first, so marker is on top)
    commands.spawn((SpatialBundle::default(), AsciiRoot, View2D));
    // Player marker as a pink '@', drawn on top
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.2, 0.6), // pink
                custom_size: Some(Vec2::splat(12.0)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)), // z=10 to draw on top
            ..Default::default()
        },
        PlayerMarker,
        View2D,
    ));
    // Force re-render on first entry
    *last2d = Last2DPos(i32::MIN, i32::MIN);
}

fn teardown_2d(
    mut commands: Commands,
    q: Query<Entity, With<View2D>>,
    mut last2d: ResMut<Last2DPos>,
) {
    for e in &q { commands.entity(e).despawn_recursive(); }
    // Force re-render next time 2D is entered
    *last2d = Last2DPos(i32::MIN, i32::MIN);
}

// Update the 3D player marker's position to match the player's world position
fn update_player_marker_3d(
    _core: Res<CoreGame>,
    mut q: Query<&mut Transform, (With<PlayerMarker>, With<View3D>)>,
) {
    if let Ok(mut transform) = q.get_single_mut() {
        // Player marker stays at origin (center of world)
        transform.translation.x = 0.0;
        transform.translation.z = 0.0;
        // Keep the cube slightly above ground
        transform.translation.y = 0.6;
    }
}

// Update visible chunks for the 3D view (centered on player, radius matches 3D render size)
fn update_visible_chunks_3d(
    core: Res<CoreGame>,
    cfg3d: Res<View3DConfig>,
    mut vis: ResMut<VisibleChunks2D>,
) {
    // Use configured chunk size and radius for 3D
    let chunk_size = cfg3d.chunk_size;
    let view_chunk_radius = cfg3d.view_chunk_radius;
    let mut center_x = 0i32;
    let mut center_y = 0i32;
    if let Some(e) = core.0.res.player_entity {
        if let Ok(pos) = core.0.world.get::<&lithicrivers_core::components::Position>(e) {
            center_x = pos.x;
            center_y = pos.y;
        }
    }
    let center_chunk = world_to_chunk_2d(center_x, center_y, chunk_size);
    let mut newset = std::collections::HashSet::new();
    for dy in -view_chunk_radius..=view_chunk_radius {
        for dx in -view_chunk_radius..=view_chunk_radius {
            newset.insert(ChunkCoord2D { cx: center_chunk.cx + dx, cy: center_chunk.cy + dy });
        }
    }
    vis.0 = newset;
}

// Ensure visible chunks are loaded for the 3D view
fn ensure_chunks_loaded_3d(
    seed: Res<WorldSeed>,
    cfg3d: Res<View3DConfig>,
    vis: Res<VisibleChunks2D>,
    mut loaded: ResMut<LoadedChunks2D>,
) {
    let chunk_size = cfg3d.chunk_size;
    for cc in vis.0.iter() {
        if !loaded.map.contains_key(cc) {
            let chunk = generate_chunk_2d(seed.0, *cc, chunk_size, chunk_size);
            loaded.map.insert(*cc, chunk);
            loaded.lru.push_back(*cc);
        }
    }
    // Evict if over capacity (skip visible ones)
    while loaded.map.len() > loaded.capacity {
        if let Some(old) = loaded.lru.pop_front() {
            if vis.0.contains(&old) {
                loaded.lru.push_back(old);
                break;
            } else {
                loaded.map.remove(&old);
            }
        } else { break; }
    }
}


// Make the 3D camera follow the fixed player marker at the origin (floating origin) and apply rotation
fn update_camera_3d_follow_player(
    _core: Res<CoreGame>,
    rot: Res<Camera3DRotation>,
    mut q: Query<&mut Transform, (With<Camera3d>, With<View3D>)>,
) {
    if let Ok(mut transform) = q.get_single_mut() {
        let r = Quat::from_axis_angle(Vec3::Y, rot.yaw)
            * Quat::from_axis_angle(Vec3::X, rot.pitch);
        let offset = r * Vec3::new(8.0, 8.0, 16.0);
        let player_origin = Vec3::ZERO;
        transform.translation = player_origin + offset;
        transform.look_at(player_origin, Vec3::Y);
    }
}


// Only update the 3D ground grid when the player moves to a new tile
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
struct Last3DPos(i32, i32);

fn update_ground_3d(
    core: Res<CoreGame>,
    _loaded: Res<LoadedChunks2D>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_ground: Query<Entity, With<Ground3D>>,
    mut last: ResMut<Last3DPos>,
) {
    let mut px = 0i32;
    let mut py = 0i32;
    if let Some(e) = core.0.res.player_entity {
        if let Ok(pos) = core.0.world.get::<&lithicrivers_core::components::Position>(e) {
            px = pos.x; py = pos.y;
        }
    }
    if (px, py) == (last.0, last.1) { info!("render_ascii_2d: unchanged player pos, skip"); return; }
    last.0 = px; last.1 = py;
    // Despawn all previous ground tiles
    for e in &q_ground { commands.entity(e).despawn_recursive(); }
    let size = 16i32; // half-extent
    let chunk_size = 32;
    for dz in -size..=size {
        for dx in -size..=size {
            let gx = px + dx;
            let gz = py + dz;
            let cc = world_to_chunk_2d(gx, gz, chunk_size);
            if let Some(chunk) = _loaded.map.get(&cc) {
                let lx = ((gx.rem_euclid(chunk_size)) as i32) as usize;
                let lz = ((gz.rem_euclid(chunk_size)) as i32) as usize;
                let idx = lz * (chunk.w as usize) + lx;
                if let Some(cell) = chunk.tiles.get(idx) {
                    let color = tile_color(cell.kind);
                    // Translate ground tile by (-player.x, 0, -player.y)
                    commands.spawn((
                        PbrBundle {
                            mesh: meshes.add(Mesh::from(bevy::prelude::shape::Box::new(0.95, 0.2, 0.95))),
                            material: materials.add(color.into()),
                            transform: Transform::from_xyz((gx - px) as f32, 0.1, (gz - py) as f32),
                            ..Default::default()
                        },
                        Ground3D,
                        View3D,
                    ));
                }
            }
        }
    }
}


fn keyboard_input_system(keys: Res<Input<KeyCode>>, mut core: ResMut<CoreGame>) {
    // Movement now uses numpad keys (cardinal + diagonal):
    // 7 8 9
    // 4 5 6
    // 1 2 3
    let mut moved = false;
    if keys.just_pressed(KeyCode::Numpad8) { core.0.queue_player_move(0, -1); moved = true; } // up
    if keys.just_pressed(KeyCode::Numpad2) { core.0.queue_player_move(0, 1); moved = true; } // down
    if keys.just_pressed(KeyCode::Numpad4) { core.0.queue_player_move(-1, 0); moved = true; } // left
    if keys.just_pressed(KeyCode::Numpad6) { core.0.queue_player_move(1, 0); moved = true; } // right
    // Diagonals
    if keys.just_pressed(KeyCode::Numpad7) { core.0.queue_player_move(-1, -1); moved = true; } // up-left
    if keys.just_pressed(KeyCode::Numpad9) { core.0.queue_player_move(1, -1); moved = true; } // up-right
    if keys.just_pressed(KeyCode::Numpad1) { core.0.queue_player_move(-1, 1); moved = true; } // down-left
    if keys.just_pressed(KeyCode::Numpad3) { core.0.queue_player_move(1, 1); moved = true; } // down-right
    if keys.just_pressed(KeyCode::Numpad5) { core.0.queue_player_move(0, 0); moved = true; } // center, same as waiting
    if moved {
        core.0.tick();
        // For now, print tick and player pos as a heartbeat.
        if let Some(e) = core.0.res.player_entity {
            if let Ok(pos) = core.0.world.get::<&lithicrivers_core::components::Position>(e) {
                info!("tick={}, player=({}, {}, {})", core.0.res.gametick, pos.x, pos.y, pos.z);
            } else {
                info!("tick={}", core.0.res.gametick);
            }
        } else {
            info!("tick={}", core.0.res.gametick);
        }
    }
}

#[allow(dead_code)]
fn tick_core_sim(time: Res<Time>, mut timer: ResMut<FixedTickTimer>, mut core: ResMut<CoreGame>) {
    if timer.tick(time.delta()).just_finished() {
        core.0.tick();
        // For now, print tick and player pos as a heartbeat.
        if let Some(e) = core.0.res.player_entity {
            if let Ok(pos) = core.0.world.get::<&lithicrivers_core::components::Position>(e) {
                info!("tick={}, player=({}, {}, {})", core.0.res.gametick, pos.x, pos.y, pos.z);
            } else {
                info!("tick={}", core.0.res.gametick);
            }
        } else {
            info!("tick={}", core.0.res.gametick);
        }
    }
}

fn toggle_view_mode(
    keys: Res<Input<KeyCode>>,
    state: Res<State<ViewMode>>,
    mut next_state: ResMut<NextState<ViewMode>>,
) {
    if keys.just_pressed(KeyCode::V) {
        let new_mode = match state.get() {
            ViewMode::TwoD => ViewMode::ThreeD,
            ViewMode::ThreeD => ViewMode::TwoD,
        };
        next_state.set(new_mode);
        info!("Switched view to {:?}", new_mode);
    }
}

// ---------------- 2D ASCII scaffold ----------------

fn load_ascii_font(mut fonts: ResMut<FontHandles>, asset_server: Res<AssetServer>) {
    if !fonts.initiated {
        // Primary: look for a repo-provided mono font at assets/fonts/monospace.ttf
        fonts.primary = asset_server.load("fonts/monospace.ttf");
        fonts.initiated = true;
        info!("Loading ASCII font: primary=assets/fonts/monospace.ttf");
    }
}

// Build a single RGBA8 atlas of ASCII glyphs (space..tilde) from the monospace.ttf
fn build_ascii_atlas(
    mut atlas: ResMut<AsciiAtlas>,
    mut images: ResMut<Assets<Image>>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    cfg: Res<View2DConfig>,
) {
    info!("build_ascii_atlas: starting (built={})", atlas.built);
    if atlas.built { return; }
    // Load font bytes directly; try common repo-relative locations
    let candidate_paths = [
        "crates/client/assets/fonts/monospace.ttf", // when running from repo root
        "assets/fonts/monospace.ttf",               // when CWD is client crate
    ];
    let mut bytes_opt: Option<Vec<u8>> = None;
    for p in candidate_paths.iter() {
        match std::fs::read(p) {
            Ok(b) => { info!("build_ascii_atlas: loaded font from {}", p); bytes_opt = Some(b); break; }
            Err(_) => { trace!("build_ascii_atlas: font not at {}", p); }
        }
    }
    let Some(bytes) = bytes_opt else {
        warn!("build_ascii_atlas: could not find monospace.ttf in expected paths; skipping atlas build");
        return;
    };
    let Ok(font) = FontArc::try_from_vec(bytes) else {
        warn!("Failed to parse monospace.ttf; skipping atlas build");
        return;
    };

    let tile = cfg.tile_px as u32;
    // Grid pack ASCII 32..=126 into 16x6 cells
    let cols = 16u32;
    let rows = 6u32;
    let cell_w = tile;
    let cell_h = tile;
    let img_w = cols * cell_w;
    let img_h = rows * cell_h;
    let mut pixels = vec![0u8; (img_w * img_h * 4) as usize];

    // Choose scale to fit height with some margin
    let scale = PxScale { x: (tile as f32) * 0.9, y: (tile as f32) * 0.9 };
    let y_baseline = (cell_h as f32) * 0.75; // adjust baseline for typical ascenders/descenders

    let mut map = HashMap::new();
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
                // White glyph with alpha (premultiplied not required here; Bevy expects straight alpha by default)
                pixels[idx + 0] = 255;
                pixels[idx + 1] = 255;
                pixels[idx + 2] = 255;
                pixels[idx + 3] = a;
            });
        }
        map.insert(ch, index);
        index += 1;
    }

    // Create Image
    let mut image = Image::new_fill(
        Extent3d { width: img_w, height: img_h, depth_or_array_layers: 1 },
        bevy::render::render_resource::TextureDimension::D2,
        &pixels,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
    );
    image.sampler = ImageSampler::nearest();
    let texture = images.add(image);
    // Build TextureAtlas (image + grid layout)
    let atlas_asset = TextureAtlas::from_grid(texture.clone(), Vec2::new(cell_w as f32, cell_h as f32), cols as usize, rows as usize, None, None);
    let atlas_handle = atlases.add(atlas_asset);

    atlas.atlas = atlas_handle;
    atlas.map = map;
    atlas.built = true;
    info!("ASCII atlas built: {}x{}, cells={}x{}", img_w, img_h, cols, rows);
}

fn world_to_chunk_2d(x: i32, y: i32, chunk: i32) -> ChunkCoord2D {
    // NOTE: chunk argument should always be CHUNK_SIZE

    let fx = div_floor(x, chunk);
    let fy = div_floor(y, chunk);
    ChunkCoord2D { cx: fx, cy: fy }
}

fn div_floor(a: i32, b: i32) -> i32 { let d = a / b; let r = a % b; if (r != 0) && ((r > 0) != (b > 0)) { d - 1 } else { d } }

fn update_visible_chunks_2d(
    core: Res<CoreGame>,
    cfg: Res<View2DConfig>,
    mut vis: ResMut<VisibleChunks2D>,
) {
    let mut center_x = 0i32;
    let mut center_y = 0i32;
    if let Some(e) = core.0.res.player_entity {
        if let Ok(pos) = core.0.world.get::<&lithicrivers_core::components::Position>(e) {
            center_x = pos.x;
            center_y = pos.y;
        }
    }
    let center_chunk = world_to_chunk_2d(center_x, center_y, cfg.chunk_size);
    let mut newset = HashSet::new();
    for dy in -cfg.view_chunk_radius..=cfg.view_chunk_radius {
        for dx in -cfg.view_chunk_radius..=cfg.view_chunk_radius {
            newset.insert(ChunkCoord2D { cx: center_chunk.cx + dx, cy: center_chunk.cy + dy });
        }
    }
    vis.0 = newset;
}

fn ensure_chunks_loaded_2d(
    seed: Res<WorldSeed>,
    cfg: Res<View2DConfig>,
    vis: Res<VisibleChunks2D>,
    mut loaded: ResMut<LoadedChunks2D>,
) {
    // Load missing visible chunks with deterministic content
    for cc in vis.0.iter() {
        if !loaded.map.contains_key(cc) {
            let chunk = generate_chunk_2d(seed.0, *cc, cfg.chunk_size, cfg.chunk_size);
            loaded.map.insert(*cc, chunk);
            loaded.lru.push_back(*cc);
        }
    }
    // Evict if over capacity (skip visible ones)
    while loaded.map.len() > loaded.capacity {
        if let Some(old) = loaded.lru.pop_front() {
            if vis.0.contains(&old) {
                // still visible; move to back and continue
                loaded.lru.push_back(old);
                break;
            } else {
                loaded.map.remove(&old);
            }
        } else { break; }
    }
}

fn generate_chunk_2d(seed: u64, cc: ChunkCoord2D, w: i32, h: i32) -> ChunkData2D {
    // NOTE: w and h should always be CHUNK_SIZE

    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    // Position-based deterministic seeding
    let mut rng = ChaCha20Rng::seed_from_u64(hash64(seed, cc.cx as i64, cc.cy as i64));
    let mut tiles = Vec::with_capacity((w*h) as usize);
    for _y in 0..h {
        for _x in 0..w {
            let r: f32 = rng.gen();
            let kind = if r < 0.10 { TileKind::Rock } // rock
                       else if r < 0.50 { TileKind::Dirt } // dirt
                       else { TileKind::Grass }; // grass
            tiles.push(TileCell { kind });
        }
    }
    ChunkData2D { w, tiles }
}

fn hash64(seed: u64, x: i64, y: i64) -> u64 {
    // Simple mix; replace with a better hash if needed
    let mut h = seed ^ 0x9E3779B97F4A7C15u64;
    h ^= (x as u64).wrapping_mul(0xBF58476D1CE4E5B9);
    h = h.rotate_left(27) ^ (y as u64).wrapping_mul(0x94D049BB133111EB);
    h ^ (h >> 33)
}

// Only rerender ASCII glyphs when the player moves to a new tile
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
struct Last2DPos(i32, i32);

fn render_ascii_2d(
    core: Res<CoreGame>,
    cfg: Res<View2DConfig>,
    _fonts: Res<FontHandles>,
    _asset_server: Res<AssetServer>,
    _loaded: Res<LoadedChunks2D>,
    mut commands: Commands,
    root_q: Query<Entity, With<AsciiRoot>>,
    mut last: ResMut<Last2DPos>,
    atlas: Res<AsciiAtlas>,
    mut grid: ResMut<AsciiGrid>,
    mut q_cells: Query<(&AsciiCell, &mut TextureAtlasSprite, &mut Transform), With<View2D>>,
    mut sprite_loader: ResMut<SpriteLoader>,
) {
    debug!("render_ascii_2d: enter, atlas_built={}, last=({}, {})", atlas.built, last.0, last.1);
    let last_blocked = core.0.res.last_blocked_tile;

    let mut px = 0i32; let mut py = 0i32;
    if let Some(e) = core.0.res.player_entity {
        if let Ok(pos) = core.0.world.get::<&lithicrivers_core::components::Position>(e) {
            px = pos.x; py = pos.y;
        }
    }
    if (px, py) == (last.0, last.1) { return; }
    last.0 = px; last.1 = py;
    // Require atlas to be built
    if !atlas.built { info!("render_ascii_2d: atlas not built yet, skip"); return; }
    let root = if let Ok(e) = root_q.get_single() { e } else { info!("render_ascii_2d: no AsciiRoot found, skip"); return };

    // Build view from core (includes entity glyph overlay)
    let view = core.0.build_view();
    let rows: i32 = view.map_lines.len() as i32;
    if rows <= 0 { info!("render_ascii_2d: empty view rows, skip"); return; }
    let cols: i32 = view.map_lines[0].chars().count() as i32;
    let half_cols = cols/2; let half_rows = rows/2;
    let start_x = view.player_pos.x - half_cols; let start_y = view.player_pos.y - half_rows;
    let sx = cfg.tile_px; let sy = cfg.tile_px;

    // Prepare computed per-cell data in row-major order
    struct CellData { index: usize, color: Color, tx: f32, ty: f32 }
    let mut computed: Vec<CellData> = Vec::with_capacity((rows*cols) as usize);
    let _span_compute = info_span!("compute_cells").entered();
    use std::collections::HashMap as StdHashMap;
    let mut char_index_cache: StdHashMap<char, usize> = StdHashMap::new();
    // Panic on any non-ASCII/unsupported glyphs so issues surface immediately (space maps to '.')
    let mut normalize = |c: char| -> char {
        match c {
            ' ' => '.', // show ground instead of invisible space
            _ if c.is_ascii() => c,
            _ => panic!(
                "Unsupported glyph encountered in render_ascii_2d::normalize: U+{:04X} '{}'",
                c as u32, c
            ),
        }
    };
    for (vy, line) in view.map_lines.iter().enumerate() {
        for (vx, ch_core) in line.chars().enumerate() {
            let vx_i = vx as i32; let vy_i = vy as i32;
            let wx = start_x + vx_i; let wy = start_y + vy_i;
            // Base from tile using sprite loader and palette
            let kind = core.0.res.world.get_tile(wx, wy);
            let tile_sprite = sprite_loader.load_sprite(kind.sprite_key(), "tiles");
            let mut ch = tile_sprite.sprites.get(0)
                .and_then(|s| s.chars().next())
                .unwrap_or('.');
            let mut color = color_for_tile(kind);
            // Overlay fluid
            if let Some(fluid) = core.0.res.fluids.get_fluid(lithicrivers_core::components::Position { x: wx, y: wy, z: 0 }) {
                ch = '~';
                color = color_for_fluid(fluid.fluid_type);
            }
            // Overlay from core map_lines (entities with Glyph etc.)
            if ch_core != ' ' {
                ch = ch_core;
            }
            // Blocked override
            if Some((wx, wy)) == last_blocked { color = Color::RED; }
            // Make sheep pop
            if ch == 's' || ch == 'S' {
                let phase = ((view.gametick % 30) as f32) / 30.0;
                let intensity = 0.7 + 0.3 * (std::f32::consts::TAU * phase).sin().abs();
                color = Color::rgb(1.0 * intensity, 0.9 * intensity, 0.2 * intensity);
            }
            let nch = normalize(ch);
            let atlas_index = *char_index_cache.entry(nch)
                .or_insert_with(|| *atlas.map.get(&nch).unwrap_or(&0usize));
            let tx = (vx_i - half_cols) as f32 * sx;
            let ty = (vy_i - half_rows) as f32 * -sy;
            computed.push(CellData { index: atlas_index, color, tx, ty });
        }
    }
    drop(_span_compute);
    // If grid not initialized or dimensions changed, (re)spawn grid once
    if !grid.initialized || grid.cols != cols || grid.rows != rows {
        info!("render_ascii_2d: (re)spawning grid {}x{}", cols, rows);
        commands.entity(root).despawn_descendants();
        commands.entity(root).with_children(|p| {
            for (i, cell) in computed.iter().enumerate() {
                p.spawn((
                    SpriteSheetBundle {
                        texture_atlas: atlas.atlas.clone(),
                        sprite: TextureAtlasSprite {
                            index: cell.index,
                            color: cell.color,
                            custom_size: Some(Vec2::splat(cfg.tile_px)),
                            ..Default::default()
                        },
                        transform: Transform::from_translation(Vec3::new(cell.tx, cell.ty, 0.0)),
                        ..Default::default()
                    },
                    AsciiCell(i),
                    View2D,
                ));
            }
        });
        grid.cols = cols; grid.rows = rows; grid.initialized = true;
        return;
    }

    // Fast path: update existing cell sprites in-place
    let _span_update = info_span!("update_cells").entered();
    for (cell, mut sprite, mut transform) in q_cells.iter_mut() {
        if let Some(cd) = computed.get(cell.0) {
            sprite.index = cd.index;
            sprite.color = cd.color;
            transform.translation.x = cd.tx;
            transform.translation.y = cd.ty;
        }
    }
    drop(_span_update);
}

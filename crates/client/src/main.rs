use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use lithicrivers_core::Game;
use lithicrivers_core::tiles::TileKind;
use std::time::Duration;
use std::collections::{HashMap, HashSet, VecDeque};
use bevy::asset::LoadState;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn div_floor_basics() {
        assert_eq!(div_floor(0, 32), 0);
        assert_eq!(div_floor(31, 32), 0);
        assert_eq!(div_floor(32, 32), 1);
        assert_eq!(div_floor(-1, 32), -1);
        assert_eq!(div_floor(-32, 32), -1);
        assert_eq!(div_floor(-33, 32), -2);
    }

    #[test]
    fn world_to_chunk_edges() {
        let c = world_to_chunk_2d(0, 0, 32);
        assert_eq!(c.cx, 0); assert_eq!(c.cy, 0);
        let c = world_to_chunk_2d(31, 31, 32);
        assert_eq!(c.cx, 0); assert_eq!(c.cy, 0);
        let c = world_to_chunk_2d(32, 0, 32);
        assert_eq!(c.cx, 1); assert_eq!(c.cy, 0);
        let c = world_to_chunk_2d(-1, -1, 32);
        assert_eq!(c.cx, -1); assert_eq!(c.cy, -1);
        let c = world_to_chunk_2d(-32, -32, 32);
        assert_eq!(c.cx, -1); assert_eq!(c.cy, -1);
        let c = world_to_chunk_2d(-33, -33, 32);
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
        let a = generate_chunk_2d(9999, cc, 32, 32);
        let b = generate_chunk_2d(9999, cc, 32, 32);
        assert_eq!(a.w, b.w);
        assert_eq!(a.tiles.len(), b.tiles.len());
        for (i, (ta, tb)) in a.tiles.iter().zip(b.tiles.iter()).enumerate() {
            assert_eq!(ta.kind as u8, tb.kind as u8, "tile mismatch at {}", i);
        }
    }

    #[test]
    fn tile_color_mapping_matches() {
        assert_eq!(tile_color(TileKind::Rock), Color::rgb(0.4, 0.4, 0.45));
        assert_eq!(tile_color(TileKind::Water), Color::rgb(0.2, 0.4, 0.8));
        assert_eq!(tile_color(TileKind::Floor), Color::rgb(0.7, 0.7, 0.7));
        assert_eq!(tile_color(TileKind::Grass), Color::rgb(0.6, 0.8, 0.6));
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

// Map shared TileKind to a Bevy Color (client-side concern)
fn tile_color(kind: TileKind) -> Color {
    match kind {
        TileKind::Rock => Color::rgb(0.4, 0.4, 0.45),
        TileKind::Water => Color::rgb(0.2, 0.4, 0.8),
        TileKind::Floor => Color::rgb(0.7, 0.7, 0.7),
        TileKind::Grass => Color::rgb(0.6, 0.8, 0.6),
    }
}

// Very small deterministic demo for 3D using the same hashing approach
fn demo_tile_at(x: i32, z: i32) -> TileKind {
    // These constants mirror the 2D generate logic but keep it simple.
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    let seed = 12345u64; // demo seed; in future, pull WorldSeed
    let cc = world_to_chunk_2d(x, z, 32);
    let local_x = (x.rem_euclid(32)) as i32;
    let local_z = (z.rem_euclid(32)) as i32;
    // Seed RNG per chunk like 2D, then advance in a stable way for local cell
    let mut rng = ChaCha20Rng::seed_from_u64(hash64(seed, cc.cx as i64, cc.cy as i64));
    // Advance RNG index by a stable offset per local cell to keep distribution consistent
    let steps = (local_z * 32 + local_x) as usize;
    for _ in 0..steps { let _: f32 = rng.gen(); }
    let r: f32 = rng.gen();
    if r < 0.10 { TileKind::Rock }
    else if r < 0.20 { TileKind::Water }
    else if r < 0.50 { TileKind::Floor }
    else { TileKind::Grass }
}

#[derive(Component)]
struct View2D;

#[derive(Component)]
struct View3D;

#[derive(Component)]
struct PlayerMarker;

// 2D ASCII view config and resources
#[derive(Resource, Debug, Clone, Copy)]
struct View2DConfig {
    tile_px: f32,
    cols: i32,
    rows: i32,
    chunk_size: i32,
    view_chunk_radius: i32,
}

#[derive(Resource, Debug, Clone, Copy)]
struct WorldSeed(pub u64);

#[derive(Component)]
struct AsciiRoot; // Parent for all ASCII glyphs so we can clear easily

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
    primary: Handle<Font>,   // assets/fonts/monospace.ttf
    fallback: Handle<Font>,  // Bevy's built-in: fonts/FiraSans-Bold.ttf
    initiated: bool,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.02, 0.02, 0.04)))
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
        .insert_resource(View2DConfig { tile_px: 16.0, cols: 80, rows: 45, chunk_size: 32, view_chunk_radius: 3 })
        .insert_resource(LoadedChunks2D { map: HashMap::new(), lru: VecDeque::new(), capacity: 256 })
        .insert_resource(VisibleChunks2D(HashSet::new()))
        .init_resource::<FontHandles>()
        .add_state::<ViewMode>()
        .add_systems(OnEnter(ViewMode::TwoD), setup_2d)
        .add_systems(OnExit(ViewMode::TwoD), teardown_2d)
        .add_systems(OnEnter(ViewMode::ThreeD), setup_3d)
        .add_systems(OnExit(ViewMode::ThreeD), teardown_3d)
        .add_systems(Startup, load_ascii_font)
        .add_systems(Update, (tick_core_sim).run_if(on_timer(Duration::from_secs_f32(1.0/60.0))))
        .add_systems(Update, keyboard_input_system)
        .add_systems(Update, toggle_view_mode)
        // 2D ASCII systems
        .add_systems(Update, (
            update_visible_chunks_2d,
            ensure_chunks_loaded_2d,
            render_ascii_2d,
        ).run_if(in_state(ViewMode::TwoD)))
        .run();
}

fn setup_2d(mut commands: Commands) {
    // Camera
    commands.spawn((Camera2dBundle::default(), View2D));
    // Simple player marker as a white sprite '@'-like placeholder
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.9, 0.9, 0.9),
                custom_size: Some(Vec2::splat(12.0)),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        },
        PlayerMarker,
        View2D,
    ));
    // Root for ASCII glyphs
    commands.spawn((SpatialBundle::default(), AsciiRoot, View2D));
}

fn teardown_2d(mut commands: Commands, q: Query<Entity, With<View2D>>) {
    for e in &q { commands.entity(e).despawn_recursive(); }
}

fn setup_3d(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Camera
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
    // Player cube
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
    // Simple demo ground of colored cubes representing tiles around origin
    let size = 16i32; // half-extent
    for gz in -size..=size {
        for gx in -size..=size {
            let kind = demo_tile_at(gx, gz);
            let color = tile_color(kind);
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(bevy::prelude::shape::Box::new(0.95, 0.2, 0.95))),
                    material: materials.add(color.into()),
                    transform: Transform::from_xyz(gx as f32, 0.1, gz as f32),
                    ..Default::default()
                },
                View3D,
            ));
        }
    }
}

fn teardown_3d(mut commands: Commands, q: Query<Entity, With<View3D>>) {
    for e in &q { commands.entity(e).despawn_recursive(); }
}

fn keyboard_input_system(keys: Res<Input<KeyCode>>, mut core: ResMut<CoreGame>) {
    // Basic movement mapping for now; will be replaced by JSON-configured actions.
    let mut moved = false;
    if keys.just_pressed(KeyCode::Up) { core.0.queue_player_move(0, -1); moved = true; }
    if keys.just_pressed(KeyCode::Down) { core.0.queue_player_move(0, 1); moved = true; }
    if keys.just_pressed(KeyCode::Left) { core.0.queue_player_move(-1, 0); moved = true; }
    if keys.just_pressed(KeyCode::Right) { core.0.queue_player_move(1, 0); moved = true; }
    if moved { /* core tick will process the intent */ }
}

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
        // Fallback: Bevy's bundled font. This path is available with default Bevy assets.
        fonts.fallback = asset_server.load("fonts/FiraSans-Bold.ttf");
        fonts.initiated = true;
        info!("Loading ASCII font: primary=assets/fonts/monospace.ttf, fallback=bevy fonts/FiraSans-Bold.ttf");
    }
}

fn world_to_chunk_2d(x: i32, y: i32, chunk: i32) -> ChunkCoord2D {
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
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    // Position-based deterministic seeding
    let mut rng = ChaCha20Rng::seed_from_u64(hash64(seed, cc.cx as i64, cc.cy as i64));
    let mut tiles = Vec::with_capacity((w*h) as usize);
    for _y in 0..h {
        for _x in 0..w {
            let r: f32 = rng.gen();
            let kind = if r < 0.10 { TileKind::Rock } // rock
                       else if r < 0.20 { TileKind::Water } // water
                       else if r < 0.50 { TileKind::Floor } // floor
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

fn render_ascii_2d(
    core: Res<CoreGame>,
    cfg: Res<View2DConfig>,
    fonts: Res<FontHandles>,
    asset_server: Res<AssetServer>,
    loaded: Res<LoadedChunks2D>,
    mut commands: Commands,
    root_q: Query<Entity, With<AsciiRoot>>,
) {
    // Choose a usable font: prefer primary if loaded, else fallback if loaded, else wait.
    let font_handle: Option<Handle<Font>> = match asset_server.get_load_state(&fonts.primary) {
        Some(LoadState::Loaded) => Some(fonts.primary.clone()),
        _ => match asset_server.get_load_state(&fonts.fallback) {
            Some(LoadState::Loaded) => Some(fonts.fallback.clone()),
            _ => None,
        },
    };
    let Some(active_font) = font_handle else { return };
    let root = if let Ok(e) = root_q.get_single() { e } else { return };
    // Clear previous children
    commands.entity(root).despawn_descendants();

    // Determine player-centered camera origin in tile space
    let mut px = 0i32; let mut py = 0i32;
    if let Some(e) = core.0.res.player_entity {
        if let Ok(pos) = core.0.world.get::<&lithicrivers_core::components::Position>(e) {
            px = pos.x; py = pos.y;
        }
    }
    let half_cols = cfg.cols/2; let half_rows = cfg.rows/2;
    let start_x = px - half_cols; let start_y = py - half_rows;
    let sx = cfg.tile_px; let sy = cfg.tile_px;
    let mut bundle = Vec::with_capacity((cfg.cols*cfg.rows) as usize);
    for vy in 0..cfg.rows {
        for vx in 0..cfg.cols {
            let wx = start_x + vx; let wy = start_y + vy;
            // Fetch tile from chunk
            let cc = world_to_chunk_2d(wx, wy, cfg.chunk_size);
            if let Some(chunk) = loaded.map.get(&cc) {
                let lx = ((wx.rem_euclid(cfg.chunk_size)) as i32) as usize;
                let ly = ((wy.rem_euclid(cfg.chunk_size)) as i32) as usize;
                let idx = ly * (chunk.w as usize) + lx;
                if let Some(cell) = chunk.tiles.get(idx) {
                    let text = Text::from_section(cell.kind.glyph().to_string(), TextStyle { font: active_font.clone(), font_size: cfg.tile_px, color: tile_color(cell.kind) })
                        .with_alignment(TextAlignment::Center);
                    let tx = (vx - half_cols) as f32 * sx;
                    let ty = (vy - half_rows) as f32 * -sy; // y-down screen
                    bundle.push((Text2dBundle {
                        text,
                        transform: Transform::from_translation(Vec3::new(tx, ty, 0.0)),
                        ..Default::default()
                    },));
                }
            }
        }
    }
    // Spawn all glyphs as children
    commands.entity(root).with_children(|p| {
        for (b,) in bundle.drain(..) { p.spawn(b); }
    });
}

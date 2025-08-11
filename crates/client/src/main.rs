use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use lithicrivers_core::Game;
use std::time::Duration;
use std::collections::{HashMap, HashSet, VecDeque};
use bevy::asset::LoadState;

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
struct TileCell { ch: char, color: Color }

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
    // Ground
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(bevy::prelude::shape::Box::new(20.0, 0.2, 20.0))),
            material: materials.add(Color::rgb(0.1, 0.2, 0.1).into()),
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
            let (ch, color) = if r < 0.10 { ('#', Color::rgb(0.4, 0.4, 0.45)) } // rock
                              else if r < 0.20 { ('~', Color::rgb(0.2, 0.4, 0.8)) } // water
                              else if r < 0.50 { ('.', Color::rgb(0.7, 0.7, 0.7)) } // floor
                              else { (',', Color::rgb(0.6, 0.8, 0.6)) }; // grass
            tiles.push(TileCell { ch, color });
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
                    let text = Text::from_section(cell.ch.to_string(), TextStyle { font: active_font.clone(), font_size: cfg.tile_px, color: cell.color })
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

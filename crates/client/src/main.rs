use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use lithicrivers_core::Game;
use std::time::Duration;

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
        .insert_resource(FixedTickTimer(Timer::from_seconds(1.0/30.0, TimerMode::Repeating)))
        .add_state::<ViewMode>()
        .add_systems(OnEnter(ViewMode::TwoD), setup_2d)
        .add_systems(OnExit(ViewMode::TwoD), teardown_2d)
        .add_systems(OnEnter(ViewMode::ThreeD), setup_3d)
        .add_systems(OnExit(ViewMode::ThreeD), teardown_3d)
        .add_systems(Update, (tick_core_sim).run_if(on_timer(Duration::from_secs_f32(1.0/60.0))))
        .add_systems(Update, keyboard_input_system)
        .add_systems(Update, toggle_view_mode)
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

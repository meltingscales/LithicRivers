use bevy::prelude::*;

#[derive(Component)]
struct Repairable;

#[derive(Resource, Default)]
struct Inventory(Vec<Option<&'static str>>);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.12)))
        .insert_resource(Inventory(vec![
            Some("Rusty Knife"), Some("Bandage"), None, None,
            Some("Scrap"), None, Some("Seed"), None,
            None, None, None, Some("Battery"),
            None, Some("Water"), None, None,
        ]))
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
            primary_window: Some(Window {
                title: "LithicRivers Demo - Inventory".into(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        })
            .set(bevy::asset::AssetPlugin { file_path: "assets".into(), ..default() })
        )
        .add_systems(Startup, setup_inventory_ui)
        .run();
}

fn setup_inventory_ui(mut commands: Commands, inv: Res<Inventory>, asset_server: Res<AssetServer>) {
    // Load a bundled monospace font from the repo's assets
    let font: Handle<Font> = asset_server.load("fonts/monospace.ttf");

    // Root node
    commands.spawn(Camera2dBundle::default());
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        background_color: Color::NONE.into(),
        ..default()
    }).with_children(|root| {
        // Panel
        root.spawn(NodeBundle {
            style: Style {
                width: Val::Px(520.0),
                height: Val::Px(420.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                column_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            background_color: Color::rgba(0.12, 0.14, 0.18, 0.95).into(),
            ..default()
        }).with_children(|panel| {
            // Title
            panel.spawn(TextBundle::from_section(
                "Inventory (demo)",
                TextStyle { font: font.clone(), font_size: 24.0, color: Color::WHITE },
            ));
            // Grid 4x4
            panel.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(340.0),
                    display: Display::Grid,
                    grid_template_columns: vec![GridTrack::fr(1.0); 4],
                    grid_template_rows: vec![GridTrack::fr(1.0); 4],
                    row_gap: Val::Px(8.0),
                    column_gap: Val::Px(8.0),
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            }).with_children(|grid| {
                for i in 0..16 {
                    let item = inv.0[i].unwrap_or("");
                    grid.spawn(NodeBundle {
                        style: Style {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::rgba(0.18, 0.2, 0.26, 1.0).into(),
                        ..default()
                    }).with_children(|slot| {
                        if !item.is_empty() {
                            slot.spawn(TextBundle::from_section(
                                item,
                                TextStyle { font: font.clone(), font_size: 18.0, color: Color::rgb(0.9, 0.9, 0.95) },
                            ));
                        } else {
                            slot.spawn(TextBundle::from_section(
                                "(empty)",
                                TextStyle { font: font.clone(), font_size: 16.0, color: Color::rgb(0.5, 0.5, 0.55) },
                            ));
                        }
                    });
                }
            });
        });
    });
}

use bevy::prelude::*;

#[derive(Resource)]
struct BodyParts(Vec<BodyPart>);

#[derive(Clone)]
struct BodyPart {
    name: &'static str,
    hp: f32,     // 0.0 - 1.0
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.12)))
        .insert_resource(BodyParts(vec![
            BodyPart { name: "Head", hp: 0.7 },
            BodyPart { name: "Torso", hp: 0.9 },
            BodyPart { name: "Left Arm", hp: 0.5 },
            BodyPart { name: "Right Arm", hp: 0.85 },
            BodyPart { name: "Left Leg", hp: 0.6 },
            BodyPart { name: "Right Leg", hp: 0.95 },
        ]))
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
            primary_window: Some(Window {
                title: "LithicRivers Demo - Body".into(),
                resolution: (900.0, 600.0).into(),
                ..default()
            }),
            ..default()
        })
            .set(bevy::asset::AssetPlugin { file_path: "assets".into(), ..default() })
        )
        .add_systems(Startup, setup_body_ui)
        .run();
}

fn setup_body_ui(mut commands: Commands, parts: Res<BodyParts>, asset_server: Res<AssetServer>) {
    // Load bundled monospace font
    let font: Handle<Font> = asset_server.load("fonts/monospace.ttf");

    commands.spawn(Camera2dBundle::default());

    // Root
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
                width: Val::Px(700.0),
                height: Val::Px(480.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            background_color: Color::rgba(0.12, 0.14, 0.18, 0.95).into(),
            ..default()
        }).with_children(|panel| {
            // Title
            panel.spawn(TextBundle::from_section(
                "Body Status (demo)",
                TextStyle { font: font.clone(), font_size: 26.0, color: Color::WHITE },
            ));

            // Column of parts
            for part in parts.0.iter().cloned() {
                panel.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(56.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(12.0),
                        ..default()
                    },
                    background_color: Color::rgba(0.16, 0.18, 0.24, 1.0).into(),
                    ..default()
                }).with_children(|row| {
                    // Name
                    row.spawn(TextBundle::from_section(
                        part.name,
                        TextStyle { font: font.clone(), font_size: 22.0, color: Color::rgb(0.9, 0.9, 0.95) },
                    ));

                    // Spacer
                    row.spawn(NodeBundle { style: Style { flex_grow: 1.0, ..default() }, ..default() });

                    // HP bar background
                    row.spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(250.0),
                            height: Val::Px(16.0),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Stretch,
                            ..default()
                        },
                        background_color: Color::rgb(0.08, 0.08, 0.1).into(),
                        ..default()
                    }).with_children(|bar| {
                        // HP fill
                        bar.spawn(NodeBundle {
                            style: Style {
                                width: Val::Px(250.0 * part.hp),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            background_color: Color::rgb(0.2, 0.8, 0.3).into(),
                            ..default()
                        });
                    });

                    // Repair button (non-functional demo)
                    row.spawn(ButtonBundle {
                        style: Style {
                            width: Val::Px(100.0),
                            height: Val::Px(32.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::rgb(0.25, 0.3, 0.4).into(),
                        ..default()
                    }).with_children(|btn| {
                        btn.spawn(TextBundle::from_section(
                            "Repair",
                            TextStyle { font: font.clone(), font_size: 18.0, color: Color::WHITE },
                        ));
                    });
                });
            }
        });
    });
}

use bevy::prelude::*;

use crate::ui::sidebar_panels::{CelestialBodyInfo, CompositionBar, HeatMapDisplay, Resources};

#[derive(Resource, Default, Debug)]
pub struct SelectedBody(pub Option<Entity>);

pub fn spawn_sidebar(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    // Root sidebar
    commands
        .spawn((
            Node {
                width: Val::Px(300.0),
                height: Val::Percent(98.0),
                position_type: PositionType::Absolute,
                left: Val::Percent(1.0),
                top: Val::Percent(1.0),

                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),

                padding: UiRect::all(Val::Px(10.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.06, 0.07, 0.5)),
        ))
        .with_children(|sidebar| {
            spawn_panel(
                sidebar, 
                &font, 
                "General info",
                |panel| {
                    panel.spawn((Text::new("No data"), CelestialBodyInfo));
                });
            spawn_panel(
                sidebar, 
                &font, 
                "Chemical composition",
                |panel| {
                    panel.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(20.0),
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                        CompositionBar
                    ));
                });
            spawn_panel(
                sidebar, 
                &font, 
                "Available molecules",
                |panel| {
                    panel.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(30.0),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                        Resources
                    ));
                });
            spawn_panel(
                sidebar,
                &font,
                "Heat map",
                |panel| {
                    panel.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(200.0),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                        HeatMapDisplay
                    ));
                });
        });
}

fn spawn_panel<F>(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    title: &str,
    content: F
) 
where
    F: FnOnce(&mut ChildSpawnerCommands)
{
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(120.0),

                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,

                border_radius: BorderRadius::all(Val::Px(8.0)),

                ..default()
            },
            BackgroundColor(Color::srgb(0.20, 0.20, 0.24)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new(title),
                TextFont {
                    font: font.clone(),
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            content(panel);
        });
}
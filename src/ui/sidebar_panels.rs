use bevy::prelude::*;

use crate::{
    resources::{molecule_table::MoleculeTable, periodic_table::PeriodicTable},
    ui::{
        camera_config::MainCamera,
        sidebar::SelectedBody
    },
    world::{
        celestial::{Celestial, Mass, ThermalBody, Velocity},
        chemistry::ChemicalComposition
    }
};

#[derive(Component, Default, Debug)]
pub struct CelestialBodyInfo;

pub fn update_body_view(
    selected: Res<SelectedBody>,
    bodies: Query<(&Transform, &Velocity, &Mass, &ThermalBody), (With<Celestial>, Without<MainCamera>)>,
    mut cam: Query<(&mut Transform, &MainCamera), With<MainCamera>>,
    mut ui_text: Query<&mut Text, With<CelestialBodyInfo>>
) {
    let Ok(mut text) = ui_text.single_mut() else { return; };

    if let Some(entity) = selected.0 {
        if let Ok((position, _vel, mass, thermal_stats)) = bodies.get(entity) {
            text.0 = format!("Mass: {:.2} kg\n Temp: {:.2} K", mass.0, thermal_stats.temperature);
            
            let Ok((mut camera_transform, camera)) = cam.single_mut() else { return; };
            camera_transform.translation = position.translation + Vec3{x: camera.dist, y: camera.dist, z: camera.dist};
            *camera_transform = camera_transform.looking_at(position.translation, Vec3::Y);
        } else {
            text.0 = format!("
                -- Nothing selected --");
            
            let Ok((mut camera_transform, camera)) = cam.single_mut() else { return; };
            camera_transform.translation = Vec3::ZERO + Vec3{x: camera.dist, y: camera.dist, z: camera.dist};
            *camera_transform = camera_transform.looking_at(Vec3::ZERO, Vec3::Y);
        }
    }
}

#[derive(Component)]
pub struct CompositionBar;

pub fn update_composition_bar(
    mut commands: Commands,
    selected: Res<SelectedBody>,
    composition_query: Query<&ChemicalComposition, With<Celestial>>,
    mut bar_query: Query<Entity, With<CompositionBar>>,
    asset_server: Res<AssetServer>,
    periodic_table: Res<PeriodicTable>,
) {
    if !selected.is_changed() {
        return;
    }

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    let Ok(bar) = bar_query.single_mut() else { return; };

    commands.entity(bar).despawn_related::<Children>();

    if let Some(entity) = selected.0 {
        if let Ok(composition) = composition_query.get(entity) {
            commands.entity(bar).with_children(|parent| {
                for element in &composition.components {
                    let mut bar_element = parent.spawn((
                        Node {
                            width: Val::Percent(element.ratio * 100.0),
                            height: Val::Percent(100.0),
                            border: UiRect::all(Val::Px(1.0)),

                            padding: UiRect::left(Val::Px(5.0)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        
                        BackgroundColor(periodic_table.get_color(element)),
                    ));
                    if element.ratio > 0.1 {
                        bar_element.with_child((
                            Text::new(periodic_table.get_name(element)),
                            TextFont {
                                font: font.clone(),
                                font_size: 10.0,
                                ..default()
                            },
                        ));
                    }
                }
            });
        }
    }
}

#[derive(Component)]
pub struct Resources;

pub fn update_resources(
    mut commands: Commands,
    selected: Res<SelectedBody>,
    composition_query: Query<&ChemicalComposition, With<Celestial>>,
    mut resource_panel: Query<Entity, With<Resources>>,
    asset_server: Res<AssetServer>,
    molecule_table: Res<MoleculeTable>,
) {
    if !selected.is_changed() {
        return;
    }

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    let Ok(panel) = resource_panel.single_mut() else { return; };

    commands.entity(panel).despawn_related::<Children>();

    if let Some(entity) = selected.0 {
        if let Ok(composition) = composition_query.get(entity) {
            commands.entity(panel).with_children(|parent| {
                let mut children_in_row = 0;
                let mut current_row = parent.spawn(
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(20.0),

                    padding: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Stretch,
                    align_items: AlignItems::Center,
                    ..default()
                });

                for molecule in &molecule_table.molecule {
                    let Some(molecule_fraction) = composition.get_max_fraction_of_molecule(molecule.id, &molecule_table) else { continue; };
                    if children_in_row == 2 {
                        current_row = parent.spawn(
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(30.0),

                            padding: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Stretch,
                            align_items: AlignItems::Center,
                            ..default()
                        });
                        children_in_row = 0;
                    }
                    current_row.with_children(|row| {
                        row.spawn(
                            Node {
                                width: Val::Percent(50.0),
                                height: Val::Percent(100.0),
                                justify_content: JustifyContent::Stretch,
                                align_items: AlignItems::Center,
                                ..default()
                        })
                        .with_children(|col| {
                            col.spawn((
                                Node {
                                    width: Val::Px(20.0),
                                    height: Val::Percent(100.0),
                                    border: UiRect::all(Val::Px(1.0)),
                                    margin: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(molecule.color),
                            ));
                            col.spawn((
                                Text::new(format!("{}: {}", molecule.name.clone(), molecule_fraction)),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 10.0,
                                    ..default()
                                },
                            ));
                        });
                    });
                    children_in_row += 1;
                }
            });
        }
    }
}
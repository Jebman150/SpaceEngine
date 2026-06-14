use bevy::{app::{App, Plugin, Startup, Update}, ecs::{schedule::{IntoScheduleConfigs, SystemSet}}};
use bevy::prelude::*;

use crate::ui::{camera_config, sidebar, sidebar_panels};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UISetup;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UIUpdate;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum UIUpdates {
    PerFrame,
    OnSelectionChange
}

pub struct UIPlugin;

fn test(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        DirectionalLight {
            illuminance: 100_000.0,
            color: Color::srgb(1.0, 0.0, 0.0),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 50.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y)
    ));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(500.0, 500.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.4, 0.4, 0.4))),
        Transform::from_xyz(0.0, -100.0, 0.0)
    ));
}

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, (
            sidebar::spawn_sidebar,
            camera_config::spawn_camera,
            test
        ).in_set(UISetup))
        .add_systems(Update, (
            (
                sidebar_panels::update_body_view
            ).in_set(UIUpdates::PerFrame),
            (
                sidebar_panels::update_composition_bar,
                sidebar_panels::update_resources,
                sidebar_panels::update_heat_map_display
            ).in_set(UIUpdates::OnSelectionChange)
        ).in_set(UIUpdate));
    }
}
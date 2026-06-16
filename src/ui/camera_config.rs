use std::f32::consts::PI;

use bevy::prelude::*;

#[derive(Component)]
pub struct MainCamera {
    pub center: Vec3,

    pub orbital_rotation: Quat,
    pub current_pitch: f32,

    pub dist: f32,
    pub min_dist: f32,
    pub max_dist: f32,
    pub zoom_strength: f32
}

pub fn spawn_camera(
    mut commands: Commands,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(200.0, 200.0, 200.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera {
            center: Vec3::ZERO,
            orbital_rotation: Quat::IDENTITY,
            current_pitch: 0.0,
            dist: 100.0,
            min_dist: 20.0,
            max_dist: 10000.0,
            zoom_strength: 5.0,
        },
        AmbientLight {
            //intensity: 100_000.0,
            color: Color::WHITE,
            brightness: 100.0,
            //shadows_enabled: true,
            //radius: 10.0,
            //range: 10000.0,
            ..default()
        },
    ));
}
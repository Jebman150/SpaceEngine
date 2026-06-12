use bevy::prelude::*;

#[derive(Component)]
pub struct MainCamera {
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
            dist: 1000.0,
            min_dist: 20.0,
            max_dist: 10000.0,
            zoom_strength: 5.0,
        }
    ));
}
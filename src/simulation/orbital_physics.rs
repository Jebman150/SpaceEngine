use bevy::{ecs::{query::{With, Without}, system::{Query, Res}}, math::Vec3, time::Time, transform::components::Transform};

use crate::world::celestial::{Celestial, Mass, Sun, Velocity};



pub fn apply_velocity(
    query: Query<(&mut Transform, &Velocity), (With<Celestial>, Without<Sun>)>,
    time: Res<Time>
) {
    for (mut transform, velocity) in query {
        transform.translation += velocity.0 * time.delta_secs();
    }
}

pub fn update_velocity(
    query: Query<(&mut Velocity, &Mass, &Transform), With<Celestial>>,
    time: Res<Time>
) {
    let bodies: Vec<(f32, Vec3)> = query
        .iter()
        .map(|(_, mass, transform)| (mass.0, transform.translation))
        .collect();

    for (mut vel, _mass, transform) in query {
        for (other_mass, other_position) in &bodies {
            let dist = transform.translation.distance(*other_position);
            let direction = (other_position - transform.translation).normalize();

            let a = if dist != 0.0 {
                direction * other_mass / dist
            } else {
                Vec3::ZERO
            };

            vel.0 += a * time.delta_secs();
        }
    }
}
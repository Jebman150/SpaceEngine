use bevy::{ecs::{query::{With, Without}, system::{Query, Res}}, math::Vec3, time::Time, transform::components::Transform};

use crate::world::celestial::{Celestial, Sun, ThermalBody};



const THERMAL_SCALE: f32 = 0.000001;

pub fn apply_temperature(
    planets: Query<(&mut ThermalBody, &Transform), (With<Celestial>, Without<Sun>)>,
    fixed_cel: Query<(&ThermalBody, &Transform), (With<Celestial>, With<Sun>)>,
    time: Res<Time>
) {
    let mut bodies: Vec<(f32, f32, Vec3)> = fixed_cel
        .iter()
        .map(|(b, t)| (b.temperature, b.emissivity, t.translation))
        .collect();

    bodies.append(
            &mut planets
            .iter()
            .map(|(b, t)| (b.temperature, b.emissivity, t.translation))
            .collect::<Vec<(f32, f32, Vec3)>>()
        );

    for (mut thermal_body, transform) in planets {
        let radiated = 
            thermal_body.emissivity
            * thermal_body.temperature.powi(4)
            * THERMAL_SCALE;

        let mut flux = 0.0;
        for (source_temp, emissivity, pos) in &bodies {
            let dist = transform.translation.distance(*pos);
            if dist < 1.0 {
                continue;
            }
            let distance_factor = 1.0 / (dist * dist);
            flux +=
                source_temp.powi(4)
                * emissivity
                * distance_factor
                * THERMAL_SCALE;
        }
        let delta_energy = (flux - radiated) * time.delta_secs();

        thermal_body.temperature += delta_energy / thermal_body.heat_capacity;
        thermal_body.temperature = thermal_body.temperature.max(0.0);
    }
}
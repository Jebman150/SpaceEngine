use bevy::{ecs::{message::MessageReader, system::Query}, input::mouse::{MouseMotion, MouseWheel}, prelude::*};

use crate::{ui::{camera_config::MainCamera, sidebar::SelectedBody}, world::celestial::Celestial};



pub fn zoom(
    mut mouse_scroll_reader: MessageReader<MouseWheel>,
    mut camera: Query<&mut MainCamera>
) {
    for mouse_wheel in mouse_scroll_reader.read() {
        let delta = mouse_wheel.y;

        for mut cam in &mut camera {
            cam.dist -= delta * cam.zoom_strength;
            cam.dist = cam.dist.min(cam.max_dist).max(cam.min_dist);
        }
    }
}

const CAMERA_SENSITIVITY: f32 = 0.002;

pub fn rotate(
    mut mouse_motion_events: MessageReader<MouseMotion>,
    mouse_clicks: Res<ButtonInput<MouseButton>>,
    mut camera: Query<&mut MainCamera>
) {
    if !mouse_clicks.pressed(MouseButton::Right) { return; }
    let Ok(mut cam) = camera.single_mut() else { return; };

    for motion in mouse_motion_events.read() {
        let delta = motion.delta;

        let right_axis = cam.orbital_rotation * Vec3::X;
        let up_axis = Vec3::Y;

        let pitch = Quat::from_axis_angle(right_axis, -delta.y * CAMERA_SENSITIVITY);
        let yaw = Quat::from_axis_angle(up_axis, -delta.x * CAMERA_SENSITIVITY);

        cam.orbital_rotation = (yaw * pitch * cam.orbital_rotation).normalize();
    }
}

pub fn center_to_selected(
    selected: Res<SelectedBody>,
    bodies: Query<&Transform, (With<Celestial>, Without<MainCamera>)>,
    mut cam: Query<&mut MainCamera>
) {
    

    let Ok(mut cam,) = cam.single_mut() else { return; };

    let Some(entity) = selected.0 else { cam.center = Vec3::ZERO; return; };
    let Ok(entity_transform) = bodies.get(entity) else { cam.center = Vec3::ZERO; return; };
    
    cam.center = entity_transform.translation;
    if !selected.is_changed() {
        return;
    }

    cam.orbital_rotation = Quat::IDENTITY;
}

pub fn update(
    mut cam: Query<(&mut Transform, &MainCamera), With<MainCamera>>
) {
    let Ok((mut camera_transform, camera)) = cam.single_mut() else { return; };

    camera_transform.translation = 
        camera.center + 
        camera.orbital_rotation *(camera.dist * Vec3::Z);
    *camera_transform = 
        camera_transform.looking_at(
            camera.center, 
            Vec3::Y
        );
    
}
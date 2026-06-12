use bevy::{ecs::{message::MessageReader, system::Query}, input::mouse::MouseWheel};

use crate::ui::camera_config::MainCamera;



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
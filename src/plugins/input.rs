use bevy::{app::{App, Plugin, Update}, ecs::schedule::{IntoScheduleConfigs, SystemSet}};

use crate::{
    input::{
        camera_control,
        selection
    }
};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputSystem;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputSystems {
    Mouse,
    Keyboard
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, (
            (
                camera_control::zoom,
                camera_control::rotate,
                camera_control::center_to_selected,
                camera_control::update
            ).in_set(InputSystems::Mouse),
            (
                selection::selected_body_change
            ).in_set(InputSystems::Keyboard)
        ).in_set(InputSystem));
    }
}
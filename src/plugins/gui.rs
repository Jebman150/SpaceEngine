use bevy::{app::{App, Plugin, Startup, Update}, ecs::schedule::{IntoScheduleConfigs, SystemSet}};

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

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, (
            sidebar::spawn_sidebar,
            camera_config::spawn_camera
        ).in_set(UISetup))
        .add_systems(Update, (
            (
                sidebar_panels::update_body_view,
            ).in_set(UIUpdates::PerFrame),
            (
                sidebar_panels::update_composition_bar,
                sidebar_panels::update_resources
            ).in_set(UIUpdates::OnSelectionChange)
        ).in_set(UIUpdate));
    }
}
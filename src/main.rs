use bevy::prelude::*;

use crate::{
    plugins::{
        gui::UIPlugin, input::InputPlugin, simulation::SimulationPlugin
    }, 
    resources::{
        molecule_table::MoleculeTable, 
        periodic_table::PeriodicTable
    }, 
    ui::{
        sidebar::SelectedBody
    }
};

pub mod input;
pub mod plugins;
pub mod simulation;
pub mod ui;
pub mod world;
pub mod resources;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<SelectedBody>()
        .init_resource::<PeriodicTable>()
        .init_resource::<MoleculeTable>()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_plugins(SimulationPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(InputPlugin)
        .configure_sets(Update, (
            plugins::simulation::SimulationSystem
                .after(plugins::input::InputSystem),
            plugins::gui::UIUpdate
                .after(plugins::simulation::SimulationSystem)
        ))
        .run();
}
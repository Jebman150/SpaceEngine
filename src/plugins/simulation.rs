use bevy::{app::{App, Plugin, Startup, Update}, ecs::schedule::{IntoScheduleConfigs, SystemSet}};

use crate::{
    simulation::
    {
        orbital_physics,
        thermal_physics
    }, world::generation
};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimulationSetup;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimulationSystem;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimulationSystems {
    OrbitalPhysics,
    ThermalPhysics
}

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, (
            generation::spawn_planets
        ).in_set(SimulationSetup))
        .add_systems(Update, (
            (
                thermal_physics::apply_temperature
            ).in_set(SimulationSystems::OrbitalPhysics),
            (
                orbital_physics::update_velocity,
                orbital_physics::apply_velocity
            ).in_set(SimulationSystems::ThermalPhysics)
        ).in_set(SimulationSystem));
    }
}
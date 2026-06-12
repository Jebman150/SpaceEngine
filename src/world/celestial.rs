use bevy::{prelude::Component, math::Vec3};

#[derive(Component, Default, Debug)]
pub struct Velocity (pub Vec3);

#[derive(Component, Default, Debug)]
pub struct Mass ( pub f32);

#[derive(Component, Debug)]
pub struct Celestial(pub String);

#[derive(Component)]
pub struct Sun;

#[derive(Component, Default, Debug)]
pub struct ThermalBody {
    pub temperature: f32,
    pub heat_capacity: f32,
    pub emissivity: f32,
}
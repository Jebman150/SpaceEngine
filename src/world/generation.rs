use std::{f32::consts::PI, fs};

use bevy::{asset::RenderAssetUsages, math::{U64Vec3, ops::cbrt}, prelude::*, render::render_resource::{Extent3d, TextureDimension, TextureFormat}};
use serde::Deserialize;

use crate::{
    resources::periodic_table::PeriodicTable, 
    world::{
        celestial::{
            Celestial,
            Velocity,
            Mass,
            Sun,
            ThermalBody
        }, 
        chemistry::ChemicalComposition
    }
};



fn calc_volume(mass: f32, chemical_composition: &ChemicalComposition, periodic_table: &PeriodicTable) -> f32 {
    let mut total_volume: f32 = 0.0;
    for element in &chemical_composition.components {
        total_volume += 
            mass * element.ratio / 
            periodic_table
                .chemical
                .iter().find(|c| c.id == element.element).expect("Cannot find element")
                .density.gas;
    }
    total_volume
}

fn get_average_color(chemical_composition: &ChemicalComposition, periodic_table: &PeriodicTable) -> Option<Color> {
    let mut sum = U64Vec3::ZERO;

    for element in &chemical_composition.components {
        let Some(chemical) = periodic_table
            .chemical
            .iter().find(|c| c.id == element.element) else { return None; };
        let element_color = chemical.color.to_srgba();

        sum.x += (element_color.red * element.ratio * 255.0) as u64;
        sum.y += (element_color.green * element.ratio * 255.0) as u64;
        sum.z += (element_color.blue * element.ratio * 255.0) as u64;
    }
    Some(Color::srgb_u8(sum.x as u8, sum.y as u8, sum.z as u8))
}

fn calc_heat_capacity(chemical_composition: &ChemicalComposition, periodic_table: &PeriodicTable) -> f32 {
    let mut sum: f32 = 0.0;

    for element in &chemical_composition.components {
        let chemical = periodic_table.get_element(element.element)
            .expect("Could not find element");
        
        sum += chemical.heat_capacity * element.ratio;
    }
    sum
}

pub fn spawn_planets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    periodic_table: Res<PeriodicTable>,
) {
    

    #[derive(Deserialize)]
    struct PlanetConfig {
        name: String,
        position: Vec3,
        velocity: Vec3,
        mass: f32,
        fixed: bool,
        temperature: f32,
        emissivity: f32,

        chemical_composition: ChemicalComposition
    }


    #[derive(Deserialize)]
    struct Config {
        planet: Vec<PlanetConfig>
    }


    let contents = fs::read_to_string("assets/config/star_system.toml").expect("Could not read initial value config file");
    let config: Config = toml::from_str(&contents).expect("Could not deserialize initial value config");

    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    for (id, planet_config) in config.planet.iter().enumerate() {
        let volume = calc_volume(planet_config.mass, &planet_config.chemical_composition, &periodic_table);
        let radius = cbrt(3.0*volume / (4.0 * PI));
        let color = get_average_color(&planet_config.chemical_composition, &periodic_table);
        let heat_capacity = calc_heat_capacity(&planet_config.chemical_composition, &periodic_table);

        let mesh = meshes.add(Sphere { radius: radius }.mesh().uv(32, 18));

        let planet_material =
        if let Some(color) = color {
            materials.add(StandardMaterial {
                base_color_texture: Some(images.add(colored_texture(color))),
                unlit: false,
                ..default()
            })
        } else {
            warn!("Could not create material for planet {} - using debug material", id);
            debug_material.clone()
        };

        if planet_config.fixed {
            commands.spawn((
                Celestial(planet_config.name.clone()),
                Sun,
                Transform::from_xyz(planet_config.position.x, planet_config.position.y, planet_config.position.z),
                PointLight {
                    intensity: planet_config.temperature,
                    color: Color::WHITE,
                    shadows_enabled: true,
                    ..default()
                },

                Velocity {0: planet_config.velocity},
                Mass (planet_config.mass),
                ThermalBody { temperature: planet_config.temperature, heat_capacity: heat_capacity, emissivity: planet_config.emissivity},

                planet_config.chemical_composition.clone(),
                Mesh3d(mesh),
                MeshMaterial3d(planet_material)
            ));
        } else {
            commands.spawn((
                Celestial(planet_config.name.clone()),
                Transform::from_xyz(planet_config.position.x, planet_config.position.y, planet_config.position.z),
                
                Velocity {0: planet_config.velocity},
                Mass (planet_config.mass),
                ThermalBody { temperature: planet_config.temperature, heat_capacity: heat_capacity, emissivity: planet_config.emissivity},

                planet_config.chemical_composition.clone(),
                Mesh3d(mesh),
                MeshMaterial3d(planet_material)
            ));
        }
    }
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn colored_texture(color: Color) -> Image {
    const TEXTURE_SIZE: usize = 8;

    let [r, g, b, a] = color.to_srgba().to_u8_array();
    let pixel = [r, g, b, a];

    let mut texture_data = vec![0u8; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for chunk in texture_data.chunks_exact_mut(4) {
        chunk.copy_from_slice(&pixel);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}
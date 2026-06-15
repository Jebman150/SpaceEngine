pub mod mesh_generation;
pub mod materials;

use std::{f32::consts::PI, fs};

use bevy::{asset::RenderAssetUsages, math::{U64Vec3, USizeVec2, ops::cbrt}, prelude::*, render::render_resource::{Extent3d, TextureDimension, TextureFormat}};
use noisy_bevy::simplex_noise_2d_seeded;
use serde::Deserialize;

use crate::{
    resources::periodic_table::PeriodicTable, 
    world::{
        celestial::{
            Celestial, HeightMap, IntensityMap, Mass, Sun, ThermalBody, Velocity
        }, chemistry::ChemicalComposition, generation::{materials::LineMaterial, mesh_generation::MeshDescriptor}, thermodynamics::HeatMap
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
    mut line_materials: ResMut<Assets<LineMaterial>>,
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

    let line_material = line_materials.add(LineMaterial {
        color: LinearRgba::GREEN,
    });

    for (id, planet_config) in config.planet.iter().enumerate() {
        // Prerequisites:
        let height_map = IntensityMap::generate(
            "Height map".to_owned(),
            USizeVec2{ x: 256, y: 256},
            &mut images,
            calc_height);

        let heat_map = IntensityMap::generate(
            "Heat map".to_owned(), 
            USizeVec2{ x: 8, y: 8}, 
            &mut images, 
            |rel_pos: Vec2| {
                let dist_from_equator = (0.5 - rel_pos.y).abs() * 2.0;
                [
                    (dist_from_equator * 255.0) as u8,
                    ((1.0 - (dist_from_equator - 0.5).abs() * 2.0) * 255.0) as u8,
                    ((1.0 - dist_from_equator) * 255.0) as u8,
                    255,
                ]
        });

        // Mesh generation:
        let volume = calc_volume(planet_config.mass, &planet_config.chemical_composition, &periodic_table);
        let radius = cbrt(3.0*volume / (4.0 * PI));
        let mesh = mesh_generation::generate_mesh(
            MeshDescriptor{
                radius: radius,
                subdivisions: 4,
                ..default()
            }, &height_map);
        //mesh.compute_normals();


        
        let color = get_average_color(&planet_config.chemical_composition, &periodic_table);
        let heat_capacity = calc_heat_capacity(&planet_config.chemical_composition, &periodic_table);

        let mesh_handle: Handle<Mesh> = meshes.add(mesh);

        let planet_material = line_material.clone();
        /*if let Some(color) = color {
            materials.add(StandardMaterial {
                base_color_texture: Some(images.add(colored_texture(color))),
                unlit: false,
                perceptual_roughness: 0.5,
                metallic: 0.0,
                ..default()
            })
        } else {
            warn!("Could not create material for planet {} - using debug material", id);
            debug_material.clone()
        };*/

        if planet_config.fixed {
            commands.spawn((
                Celestial(planet_config.name.clone()),
                Sun,
                Transform::from_xyz(planet_config.position.x, planet_config.position.y, planet_config.position.z),

                Velocity {0: planet_config.velocity},
                Mass (planet_config.mass),
                ThermalBody { temperature: planet_config.temperature, heat_capacity: heat_capacity, emissivity: planet_config.emissivity},

                HeatMap(heat_map),
                HeightMap(height_map),

                planet_config.chemical_composition.clone(),
                Mesh3d(mesh_handle),
                MeshMaterial3d(planet_material)
            ));
        } else {
            commands.spawn((
                Celestial(planet_config.name.clone()),
                Transform::from_xyz(planet_config.position.x, planet_config.position.y, planet_config.position.z),
                
                Velocity {0: planet_config.velocity},
                Mass (planet_config.mass),
                ThermalBody { temperature: planet_config.temperature, heat_capacity: heat_capacity, emissivity: planet_config.emissivity},
                
                HeatMap(heat_map),
                HeightMap(height_map),

                planet_config.chemical_composition.clone(),
                Mesh3d(mesh_handle),
                MeshMaterial3d(planet_material)
            ));
        }
    }
}

fn calc_height(pos: Vec2) -> [u8; 4] {
    let val = (simplex_noise_2d_seeded(pos * 10.0, 1.0) * 255.0) as u8;
    [
        val,
        val,
        val,
        255
    ]
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
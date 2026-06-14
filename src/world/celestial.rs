use bevy::{asset::RenderAssetUsages, math::USizeVec2, prelude::*, render::render_resource::{Extent3d, TextureDimension, TextureFormat}};

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

#[derive(Component, Clone, Default)]
pub struct HeightMap ( pub IntensityMap );

#[derive(Clone, Default)]
pub struct IntensityMap {
    pub name: String,
    pub handle: Handle<Image>
}

impl IntensityMap {
    pub fn generate(name: String, resolution: USizeVec2, images: &mut ResMut<Assets<Image>>, evaluate: impl Fn(Vec2) -> [u8; 4]) -> Self {
        let mut data: Vec<u8> = vec![0; resolution.x * resolution.y * 4];
        for i in 0..resolution.x {
            for j in 0..resolution.y {
                let rel_coord = Vec2{
                    x: i as f32 / resolution.x as f32,
                    y: j as f32 / resolution.y as f32,
                };
                let index = (j*resolution.x + i) * 4;
                data[index..index+4].copy_from_slice(&evaluate(rel_coord));
            }
        }
        let image = Image::new(
            Extent3d {
                width: resolution.x as u32,
                height: resolution.y as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        );

        let handle = images.add(image);

        Self {
            name: name,
            handle: handle
        }
    }
}
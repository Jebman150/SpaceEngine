use bevy::{prelude::*, render::render_resource::AsBindGroup, shader::ShaderRef};

const SHADER_ASSET_PATH: &str = "shaders/line_material.wgsl";

#[derive(Asset, TypePath, Default, AsBindGroup, Debug, Clone)]
pub struct LineMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
}

impl Material for LineMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}
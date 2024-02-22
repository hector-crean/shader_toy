use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};

// This struct defines the data that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CustomMaterial {
    #[uniform(0)]
    pub color: Color,
    #[texture(1)]
    #[sampler(2)]
    pub albedo_texture: Option<Handle<Image>>,
    #[texture(3)]
    #[sampler(4)]
    pub ao_texture: Option<Handle<Image>>,
    #[texture(5)]
    #[sampler(6)]
    pub normal_texture: Option<Handle<Image>>,
    #[texture(7)]
    #[sampler(8)]
    pub game_of_life_texture: Option<Handle<Image>>,

    pub alpha_mode: AlphaMode,
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

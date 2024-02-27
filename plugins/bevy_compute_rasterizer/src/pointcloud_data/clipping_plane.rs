use bevy::{
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{ShaderType, UniformBuffer},
        Extract,
    },
};

#[derive(Clone, Component, Debug, ShaderType)]
pub struct ClippingPlaneRange {
    /// The minimum (signed) distance from a visible point's centroid to the plane.
    pub min_sdist: f32,
    /// The maximum (signed) distance from a visible point's centroid to the plane.
    pub max_sdist: f32,
}

impl Default for ClippingPlaneRange {
    fn default() -> Self {
        Self {
            min_sdist: 0.0,
            max_sdist: f32::INFINITY,
        }
    }
}

#[derive(Bundle, Default)]
pub struct ClippingPlaneBundle {
    pub range: ClippingPlaneRange,
    pub transform: TransformBundle,
}

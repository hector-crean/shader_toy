use bevy::{
    prelude::*,
    render::{
        render_resource::{ShaderType, UniformBuffer},
        Extract,
    },
};

use super::clipping_plane::ClippingPlaneRange;

#[derive(Clone, Component, Debug, Default, ShaderType)]
pub(crate) struct GpuClippingPlaneRange {
    pub origin: Vec3,
    pub unit_normal: Vec3,
    pub min_sdist: f32,
    pub max_sdist: f32,
}

#[derive(Debug, Default, ShaderType)]
pub(crate) struct GpuClippingPlaneRanges {
    pub ranges: [GpuClippingPlaneRange; MAX_CLIPPING_PLANES],
    pub num_ranges: u32,
}

/// The clipping shader is `O(planes * points)`, so we set a reasonable limit.
pub const MAX_CLIPPING_PLANES: usize = 16;

#[derive(Resource, Default)]
pub struct GpuClippingPlaneRangesUniformBuffer(pub UniformBuffer<GpuClippingPlaneRanges>);

impl GpuClippingPlaneRanges {
    pub fn extract(
        clipping_planes: Extract<Query<(&ClippingPlaneRange, &GlobalTransform)>>,
        mut clipping_plane_uniform: ResMut<GpuClippingPlaneRangesUniformBuffer>,
    ) {
        let mut iter = clipping_planes.iter();
        let mut gpu_planes = GpuClippingPlaneRanges::default();
        for (range, transform) in iter.by_ref() {
            let (_, rotation, translation) = transform.to_scale_rotation_translation();
            gpu_planes.ranges[gpu_planes.num_ranges as usize] = GpuClippingPlaneRange {
                origin: translation,
                unit_normal: rotation * Vec3::X,
                min_sdist: range.min_sdist,
                max_sdist: range.max_sdist,
            };
            gpu_planes.num_ranges += 1;
            if gpu_planes.num_ranges as usize == MAX_CLIPPING_PLANES {
                break;
            }
        }
        if iter.next().is_some() {
            warn!(
            "Too many GpuClippingPlaneRanges entities, at most {MAX_CLIPPING_PLANES} are supported"
        );
        }
        clipping_plane_uniform.0.set(gpu_planes);
    }
    pub fn prepare(
        render_device: Res<bevy::render::renderer::RenderDevice>,
        render_queue: Res<bevy::render::renderer::RenderQueue>,
        mut clipping_plane_uniform: ResMut<GpuClippingPlaneRangesUniformBuffer>,
    ) {
        // Values already pushed in extract stage.
        clipping_plane_uniform
            .0
            .write_buffer(&render_device, &render_queue);
    }
}

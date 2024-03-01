use bevy::{
    core::cast_slice,
    ecs::{
        query::ROQueryItem,
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    prelude::*,
    reflect::TypePath,
    render::{
        extract_component::{ComponentUniforms, DynamicUniformIndex, UniformComponentPlugin},
        render_asset::{RenderAsset, RenderAssetPlugin, RenderAssets},
        render_phase::{PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass},
        render_resource::*,
        renderer::RenderDevice,
        texture::BevyDefault,
        view::{ViewUniform, ViewUniforms},
        Extract, Render, RenderApp, RenderSet,
    },
};

use crate::bindings::{
    vector_field_3d::VectorField3D, voxels::Voxels, voxels_settings::VoxelsSettings,
};

#[derive(Bundle)]
pub struct VoxelsBundle {
    pub voxels: Voxels,
    pub vector_field_3d: Handle<VectorField3D>,
}

#[derive(Resource)]
pub struct VoxelsPipeline {
    vector_field_3d_bind_group_layout: BindGroupLayout,
    settings_bind_group_layout: BindGroupLayout,
    raymarched_mesh_bind_group_layout: BindGroupLayout,
}

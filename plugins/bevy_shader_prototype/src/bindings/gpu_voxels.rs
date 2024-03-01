use bevy::{
    ecs::{
        storage,
        system::lifetimeless::{Read, SRes},
    },
    prelude::*,
    render::{
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        gpu_component_array_buffer,
        render_asset::{RenderAsset, RenderAssetUsages, RenderAssets},
        render_graph::{self, RenderGraph, RenderLabel},
        render_phase::{PhaseItem, RenderCommand, RenderCommandResult},
        render_resource::{
            binding_types::{storage_buffer, uniform_buffer},
            encase::UniformBuffer,
            *,
        },
        renderer::{RenderContext, RenderDevice},
        view::{ExtractedView, ViewUniforms},
        Extract, Render, RenderApp, RenderSet,
    },
    utils::HashMap,
    window::WindowPlugin,
};
use bytemuck::{Pod, Zeroable};
use std::{borrow::Cow, marker::PhantomData, ops::Deref};

use super::voxels::{self, Voxels};

#[derive(Bundle)]
struct VoxelBundle {
    handle: Handle<Voxels>,
}

impl VoxelBundle {
    pub fn spawn(mut commands: Commands, mut voxel_res: ResMut<Assets<Voxels>>) {
        //    commands.spawn((
        //     VoxelBundle {
        //         voxels_handle: voxel_res.add()
        //     }
        //    ))
    }
}
/*
RenderAssetPlugin extracts the changed assets from the “app world” into the “render world” and prepares them for the GPU.
They can then be accessed from the RenderAssets resource.

We have
*/

#[derive(Clone, Resource)]
pub struct VoxelsPipeline {
    pub shader: Handle<Shader>,
    pub voxels_bind_group_layout: BindGroupLayout,
}

pub struct GpuVoxels {
    voxels_buffer: Buffer,
    voxels_count: usize,
}

impl RenderAsset for Voxels {
    type Param = SRes<RenderDevice>;
    type PreparedAsset = GpuVoxels;

    fn asset_usage(&self) -> RenderAssetUsages {
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD
    }
    fn prepare_asset(
        self,
        (render_device): &mut bevy::ecs::system::SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, bevy::render::render_asset::PrepareAssetError<Self>> {
        let voxels_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("voxels buffer"),
            contents: bytemuck::cast_slice(self.voxels.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST | BufferUsages::STORAGE,
        });

        let voxels_count = self.voxels.len();

        Ok(Self::PreparedAsset {
            voxels_buffer,
            voxels_count,
        })
    }
}

#[derive(Resource)]
pub struct VoxelBindGroup {
    pub value: BindGroup,
}
pub fn prepare_voxel_bind_group(
    mut commands: Commands,
    voxels_pipeline: Res<VoxelsPipeline>,
    render_device: Res<RenderDevice>,
    voxels_uniforms: Res<ComponentUniforms<VoxelsUniform>>,
) {
    if let Some(binding) = voxels_uniforms.uniforms().binding() {
        commands.insert_resource(VoxelBindGroup {
            value: render_device.create_bind_group(
                Some("polyline_bind_group"),
                &voxels_pipeline.voxels_bind_group_layout,
                &[BindGroupEntry {
                    binding: 0,
                    resource: binding,
                }],
            ),
        });
    }
}

pub struct SetVoxelBindGroup<const I: usize>;
impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetVoxelBindGroup<I> {
    type Param = SRes<VoxelBindGroup>;
    type ViewQuery = ();
    type ItemQuery = Read<DynamicUniformIndex<VoxelsUniform>>;

    fn render<'w>(
        item: &P,
        view: bevy::ecs::query::ROQueryItem<'w, Self::ViewQuery>,
        voxels_idx: Option<bevy::ecs::query::ROQueryItem<'w, Self::ItemQuery>>,
        bind_group: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(voxels_idx) = voxels_idx {
            pass.set_bind_group(I, &bind_group.into_inner().value, &[voxels_idx.index()]);
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

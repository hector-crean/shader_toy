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
        render_asset::{RenderAsset, RenderAssetPlugin, RenderAssetUsages, RenderAssets},
        render_graph::{self, RenderGraph, RenderLabel},
        render_phase::{PhaseItem, RenderCommand, RenderCommandResult},
        render_resource::{
            binding_types::{sampler, storage_buffer, texture_3d, uniform_buffer},
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

#[derive(ShaderType, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Triangle {
    a: Vec3,
    b: Vec3,
    c: Vec3,
}

#[derive(ShaderType, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Cube {
    triangle_count: u32,
    triangles: [Triangle; 5],
}

#[derive(Clone, Asset, TypePath, Component)]
#[repr(C)]
pub struct Grid {
    pub cubes: Vec<Cube>,
}

pub struct GpuGrid {
    grid_buffer: Buffer,
    grid_count: usize,
}

impl RenderAsset for Grid {
    type Param = SRes<RenderDevice>;
    type PreparedAsset = GpuGrid;

    fn asset_usage(&self) -> RenderAssetUsages {
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD
    }
    fn prepare_asset(
        self,
        (render_device): &mut bevy::ecs::system::SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, bevy::render::render_asset::PrepareAssetError<Self>> {
        let grid_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("voxels buffer"),
            contents: bytemuck::cast_slice(self.cubes.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST | BufferUsages::STORAGE,
        });

        let grid_count = self.cubes.len();

        Ok(Self::PreparedAsset {
            grid_buffer,
            grid_count,
        })
    }
}

#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
pub struct VolumetricRenderingSettings {
    iso: f32,
    // WebGL2 structs must be 16 byte aligned.
    #[cfg(feature = "webgl2")]
    _webgl2_padding: Vec3,
}

#[derive(Component)]
pub struct VolumetricRenderingBindGroup {
    pub value: BindGroup,
}
pub fn prepare_volumetric_rendering_bind_group(
    mut commands: Commands,
    volumetric_compute_pipeline: Res<VolumetricComputePipeline>,
    render_device: Res<RenderDevice>,
    volumetric_settings: Res<ComponentUniforms<VolumetricRenderingSettings>>,
    gpu_grids: ResMut<RenderAssets<Grid>>,
    gpu_images: Res<RenderAssets<Image>>,
    query: Query<(
        Entity,
        &Handle<Image>,
        &Handle<Grid>,
        &DynamicUniformIndex<VolumetricRenderingSettings>,
    )>,
) {
    let Some(volumetric_settings) = volumetric_settings.uniforms().binding() else {
        return;
    };

    for (entity, gpu_image_handle, gpu_grid_handle, settings_idx) in query.iter() {
        let Some(gpu_image) = gpu_images.get(gpu_image_handle) else {
            continue;
        };
        let Some(gpu_grid) = gpu_grids.get(gpu_grid_handle) else {
            continue;
        };

        let bind_group = VolumetricRenderingBindGroup {
            value: render_device.create_bind_group(
                Some("polyline_bind_group"),
                &volumetric_compute_pipeline.bind_group_0_layout,
                &[
                    BindGroupEntry {
                        binding: 0,
                        resource: gpu_image.texture_view.into_binding(),
                    },
                    //How do we get just the index that we want? I think only later when we're
                    //doing the draw commands do we specify:
                    // i.e. pass.set_bind_group(I, bind_group_value, &[idx])
                    //https://github.com/cessationoftime/bevy_plot/blob/e1a7be92615d7089aeea8244df9e580e47aaca20/src/markers/mod.rs
                    BindGroupEntry {
                        binding: 1,
                        resource: volumetric_settings.clone(),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: gpu_grid.grid_buffer.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: volumetric_settings.clone(),
                    },
                ],
            ),
        };

        commands.entity(entity).insert(bind_group);
    }
}

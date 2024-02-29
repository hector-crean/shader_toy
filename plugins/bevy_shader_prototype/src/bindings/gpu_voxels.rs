use bevy::{
    ecs::{
        storage,
        system::lifetimeless::{Read, SRes},
    },
    prelude::*,
    render::{
        extract_component::{
            ComponentUniforms, ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin,
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

use super::voxels::Voxels;

#[derive(Component)]
pub struct GpuVoxels {
    buf: Buffer,
    len: usize,
}


impl ExtractComponent for Voxels {
    type QueryData: ReadOnlyQueryData;
    type QueryFilter: QueryFilter;
    type Out = GpuVoxels;

    fn extract_component(item: bevy::ecs::query::QueryItem<'_, Self::QueryData>) -> Option<Self::Out> {
        let buf = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("voxels buffer"),
            contents: bytemuck::cast_slice(self.voxels.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST | BufferUsages::STORAGE,
        });

        let len = self.voxels.len();

        Ok(Self::PreparedAsset { buf, len })
    }

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
        let buf = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("voxels buffer"),
            contents: bytemuck::cast_slice(self.voxels.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST | BufferUsages::STORAGE,
        });

        let len = self.voxels.len();

        Ok(Self::PreparedAsset { buf, len })
    }
}

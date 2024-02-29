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

/**
 * We have a 'volumetric material component'
 */

#[derive(ShaderType, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Triangle {
    a: Vec3,
    b: Vec3,
    c: Vec3,
}

#[derive(ShaderType, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Voxel {
    triangle_count: u32,
    triangles: [Triangle; 5],
}

#[derive(Asset, Clone, TypePath, Component)]
pub struct Voxels {
    pub voxels: Vec<Voxel>,
}

pub mod render;

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
use render::{
    bindings::{prepare_volumetric_rendering_bind_group, Grid, VolumetricRenderingSettings},
    compute_pipeline::{PolygoniseMeshComputeNode, PolygoniseMeshComputePipeline},
};
use std::{borrow::Cow, marker::PhantomData, ops::Deref};

pub struct VolumetricRenderingPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct VolumetricRenderingLabel;

impl Plugin for VolumetricRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RenderAssetPlugin::<Grid>::default(),
            ExtractComponentPlugin::<VolumetricRenderingSettings>::default(),
            UniformComponentPlugin::<VolumetricRenderingSettings>::default(),
        ));

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            prepare_volumetric_rendering_bind_group.in_set(RenderSet::PrepareBindGroups),
        );

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node(
            VolumetricRenderingLabel,
            PolygoniseMeshComputeNode::default(),
        );
        render_graph.add_node_edge(
            VolumetricRenderingLabel,
            bevy::render::graph::CameraDriverLabel,
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<PolygoniseMeshComputePipeline>();
    }
}

#[derive(Bundle)]
struct VolumetricRenderingBundle {
    // This will likely be an MRI scan, or other volumetric data
    vector_field_3d: Handle<Image>,
    // The isosurface etc.
    settings: VolumetricRenderingSettings,
    // We compute the 'marching cubes' buffer
    marching_cubes_grid_handle: Handle<Grid>,
    // marching_cubes_mesh_handle: Handle<Mesh>,
}

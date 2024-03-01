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

#[derive(Clone, Resource)]
pub struct PolygoniseMeshComputePipeline {
    pub shader: Handle<Shader>,
    pub init_pipeline_id: CachedComputePipelineId,
    pub update_pipeline_id: CachedComputePipelineId,
    pub bind_group_0_layout: BindGroupLayout,
}

pub fn compute_pipeline_descriptor(
    shader: Handle<Shader>,
    entry_point: &str,
    bind_group_layout: &BindGroupLayout,
) -> ComputePipelineDescriptor {
    ComputePipelineDescriptor {
        label: None,
        layout: vec![bind_group_layout.clone()],
        shader,
        shader_defs: vec![],
        entry_point: Cow::from(entry_point.to_owned()),
        push_constant_ranges: vec![],
    }
}

impl FromWorld for PolygoniseMeshComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        // We need to define the bind group layout used for our pipeline
        let bind_group_0_layout = render_device.create_bind_group_layout(
            "volumetric_rendering_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                // The layout entries will only be visible in the fragment stage
                ShaderStages::COMPUTE,
                (
                    texture_3d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<VolumetricRenderingSettings>(true),
                    storage_buffer::<Cube>(false),
                ),
            ),
        );

        // We can create the sampler here since it won't change at runtime and doesn't depend on the view
        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        // Get the shader handle
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/post_processing.wgsl");

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();

        let init_pipeline_id = pipeline_cache.queue_compute_pipeline(compute_pipeline_descriptor(
            shader.clone(),
            "init",
            &bind_group_0_layout,
        ));

        let update_pipeline_id = pipeline_cache.queue_compute_pipeline(
            compute_pipeline_descriptor(shader, "update", &bind_group_0_layout),
        );

        Self {
            shader,
            bind_group_0_layout,
            init_pipeline_id,
            update_pipeline_id,
        }
    }
}

enum PolygoniseMeshState {
    Loading,
    Init,
    Update,
}

pub struct PolygoniseMeshComputeNode {
    state: PolygoniseMeshState,
}

impl Default for PolygoniseMeshComputeNode {
    fn default() -> Self {
        Self {
            state: PolygoniseMeshState::Loading,
        }
    }
}

impl render_graph::Node for PolygoniseMeshComputeNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<PolygoniseMeshComputePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            PolygoniseMeshState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline_id)
                {
                    self.state = PolygoniseMeshState::Init;
                }
            }
            PolygoniseMeshState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline_id)
                {
                    self.state = PolygoniseMeshState::Update;
                }
            }
            PolygoniseMeshState::Update => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let texture_bind_group = &world.resource::<GameOfLifeImageBindGroup>().0;

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<PolygoniseMeshComputePipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        //this is where we set the correct offset for the UniformIndexedBuffer...
        pass.set_bind_group(0, texture_bind_group, &[]);

        // select the pipeline based on the current state
        match self.state {
            PolygoniseMeshState::Loading => {}
            PolygoniseMeshState::Init => {
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.init_pipeline_id)
                    .unwrap();
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
            }
            PolygoniseMeshState::Update => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline_id)
                    .unwrap();
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
            }
        }

        Ok(())
    }
}

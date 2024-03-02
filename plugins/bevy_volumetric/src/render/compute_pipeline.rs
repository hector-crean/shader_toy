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
    utils::hashbrown::HashMap,
    window::WindowPlugin,
};
use bytemuck::{Pod, Zeroable};
use std::{borrow::Cow, marker::PhantomData, ops::Deref};

use crate::VolumetricRenderingBundle;

use super::bindings::{Cube, Grid, VolumetricRenderingSettings};

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

        let pipeline_cache = world.resource_mut::<PipelineCache>();

        let init_pipeline_id = pipeline_cache.queue_compute_pipeline(compute_pipeline_descriptor(
            shader.clone(),
            "init",
            &bind_group_0_layout,
        ));

        let update_pipeline_id = pipeline_cache.queue_compute_pipeline(
            compute_pipeline_descriptor(shader.clone(), "update", &bind_group_0_layout),
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
    states: HashMap<Entity, PolygoniseMeshState>,
    volumetric_mesh_systems: QueryState<
        Entity,
        (
            With<VolumetricRenderingSettings>,
            With<Handle<Grid>>,
            With<Handle<Image>>,
        ),
    >,
}

impl PolygoniseMeshComputeNode {
    pub fn new(world: &mut World) -> Self {
        Self {
            volumetric_mesh_systems: QueryState::new(world),
            states: HashMap::default(),
        }
    }
    fn update_state(
        &mut self,
        entity: Entity,
        pipeline_cache: &PipelineCache,
        pipeline: &PolygoniseMeshComputePipeline,
    ) {
        let update_state = match self.states.get(&entity) {
            Some(state) => state,
            None => {
                self.states.insert(entity, PolygoniseMeshState::Loading);
                &PolygoniseMeshState::Loading
            }
        };

        match update_state {
            PolygoniseMeshState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline_id)
                {
                    self.states.insert(entity, PolygoniseMeshState::Init);
                }
            }
            PolygoniseMeshState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline_id)
                {
                    self.states.insert(entity, PolygoniseMeshState::Update);
                }
            }
            PolygoniseMeshState::Update => {}
        }
    }
    pub fn run_compute_pass(
        render_context: &mut RenderContext,
        bind_group: &BindGroup,
        pipeline_cache: &PipelineCache,
        pipeline: CachedComputePipelineId,
    ) {
        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, bind_group, &[]);

        let pipeline = pipeline_cache.get_compute_pipeline(pipeline).unwrap();
        pass.set_pipeline(pipeline);

        pass.dispatch_workgroups(PARTICLE_COUNT / WORKGROUP_SIZE, 1, 1);
    }
}

impl render_graph::Node for PolygoniseMeshComputeNode {
    fn update(&mut self, world: &mut World) {
        let mut systems = world.query_filtered::<Entity, (
            With<VolumetricRenderingSettings>,
            With<Handle<Grid>>,
            With<Handle<Image>>,
        )>();

        let pipeline = world.resource::<PolygoniseMeshComputePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        for entity in systems.iter(world) {
            // if the corresponding pipeline has loaded, transition to the next stage
            self.update_state(entity, pipeline_cache, pipeline);
        }
        //Update the query for the run step
        self.volumetric_mesh_systems.update_archetypes(world);
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<PolygoniseMeshComputePipeline>();

        for entity in self.volumetric_mesh_systems.iter_manual(world) {
            // select the pipeline based on the current state
            if let Some(pipeline) = match self.states[&entity] {
                PolygoniseMeshState::Loading => None,
                PolygoniseMeshState::Init => Some(pipeline.init_pipeline_id),
                PolygoniseMeshState::Update => Some(pipeline.update_pipeline_id),
            } {
                Self::run_compute_pass(
                    render_context,
                    &particle_systems_render.update_bind_group[&entity],
                    pipeline_cache,
                    pipeline,
                );
            }
        }

        Ok(())
    }
}

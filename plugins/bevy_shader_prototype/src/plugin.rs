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
        render_asset::{RenderAsset, RenderAssetPlugin, RenderAssetUsages, RenderAssets},
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

use crate::bindings::{
    vector_field_3d::VectorField3D, voxels::Voxels, voxels_settings::VoxelsSettings,
};

#[derive(Resource)]
pub struct VolumetricMaterialPipeline {
    vector_field_3d_bind_group_layout: BindGroupLayout,
    settings_bind_group_layout: BindGroupLayout,
    raymarched_mesh_bind_group_layout: BindGroupLayout,
}

impl Plugin for VolumetricMaterialPipeline {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RenderAssetPlugin::<Voxels>::default(),
            ExtractComponentPlugin::<VectorField3D>::default(),
            ExtractComponentPlugin::<VoxelsSettings>::default(),
            UniformComponentPlugin::<VoxelsSettings>::default(),
        ));

        let render_app = app.sub_app_mut(RenderApp);

        render_app.add_systems(
            Render,
            RaymarchedMeshBindGroup::prepare.in_set(RenderSet::PrepareBindGroups),
        );

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node(VolumetricNodeLabel, VolumetricNode::default());
        render_graph.add_node_edge(VolumetricNodeLabel, bevy::render::graph::CameraDriverLabel);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<MarchingCubesComputePipeline>();
    }
}

// #[derive(Component, Debug)]
// pub struct MarchingCubesViewBindGroup {
//     value: BindGroup,
// }

// impl MarchingCubesViewBindGroup {
//     pub fn prepare(
//         mut commands: Commands,
//         render_device: Res<RenderDevice>,
//         marching_cubes_compute_pipeline: Res<MarchingCubesComputePipeline>,
//         view_uniforms: Res<ViewUniforms>,
//         views: Query<Entity, With<ExtractedView>>,
//     ) {
//         if let Some(view_binding) = view_uniforms.uniforms.binding() {
//             for entity in views.iter() {
//                 let view_bind_group = render_device.create_bind_group(
//                     "shape_view_bind_group",
//                     &marching_cubes_compute_pipeline.view_layout,
//                     &BindGroupEntries::single(view_binding.clone()),
//                 );

//                 commands.entity(entity).insert(Self {
//                     value: view_bind_group,
//                 });
//             }
//         }
//     }
// }

#[derive(Component)]
pub struct RaymarchedMeshBindGroup(BindGroup);

impl Deref for RaymarchedMeshBindGroup {
    type Target = BindGroup;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl RaymarchedMeshBindGroup {
    fn layout(device: &RenderDevice) -> BindGroupLayout {
        device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::single(ShaderStages::all(), storage_buffer::<Cube>(false)),
        )
    }

    fn new(
        device: &RenderDevice,
        layout: &BindGroupLayout,
        gpu_raymarched_mesh: &GpuRaymarchedMesh,
    ) -> Self {
        let bind_group = device.create_bind_group(
            None,
            layout,
            &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &gpu_raymarched_mesh.buf,
                    offset: 0,
                    size: None,
                }),
            }],
        );

        Self(bind_group)
    }
    fn prepare(
        mut commands: Commands,
        pipeline: Res<VolumetricMaterialPipeline>,
        gpu_images: Res<RenderAssets<Image>>,
        raymarched_meshes: Res<RenderAssets<RaymarchedMesh>>,
        volumetric_settings: Res<ComponentUniforms<VolumetricMaterialSettings>>,
        volumetric_material_query: Query<(Entity, &Handle<RaymarchedMesh>, &VectorField3D)>,
        render_device: Res<RenderDevice>,
    ) {
        for (entity, raymarched_mesh, vector_field) in volumetric_material_query.iter() {
            match (
                gpu_images.get(&vector_field.vector_field_texture_3d),
                raymarched_meshes.get(raymarched_mesh),
            ) {
                (Some(gpu_image), Some(gpu_raymarched_mesh)) => {
                    let bind_group_layout = RaymarchedMeshBindGroup::layout(&render_device);

                    let raymarched_bind_group = RaymarchedMeshBindGroup::new(
                        &render_device,
                        &bind_group_layout,
                        &gpu_raymarched_mesh,
                    );

                    commands.entity(entity).insert(raymarched_bind_group);
                }
                _ => {}
            }
        }
    }
}

pub struct SetRaymarchedMeshBindGroup<const I: usize>;

impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetRaymarchedMeshBindGroup<I> {
    type Param = ();
    type ViewQuery = ();
    type ItemQuery = (Has<RaymarchedMeshBindGroup>);

    fn render<'w>(
        item: &P,
        (raymarch_mesh_bind_group): bevy::ecs::query::ROQueryItem<'w, Self::ViewQuery>,
        entity: Option<bevy::ecs::query::ROQueryItem<'w, Self::ItemQuery>>,
        param: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> bevy::render::render_phase::RenderCommandResult {
        pass.set_bind_group(I, &item, &[]);
        RenderCommandResult::Success
    }
}

// This is the component that will get passed to the shader
#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
struct PostProcessSettings {
    intensity: f32,
    // WebGL2 structs must be 16 byte aligned.
    #[cfg(feature = "webgl2")]
    _webgl2_padding: Vec3,
}

struct PostProcessSettingsBindGroup<T> {
    pub value: BindGroup,
    _marker: PhantomData<T>,
}

impl PostProcessSettingsBindGroup {
    fn prepare(
        mut commands: Commands,
        render_device: Res<RenderDevice>,
        post_process_pipeline: Res<PostProcessPipeline>,
        // pipeline_cache: Res<PipelineCache>,
        settings_uniforms: Res<ComponentUniforms<PostProcessSettings>>,
    ) -> Result<(), _> {
        // Get the pipeline from the cache
        // let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id)
        // else {
        //     return Ok(());
        // };

        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let bind_group = render_device.create_bind_group(
            None,
            &post_process_pipeline.layout,
            // It's important for this to match the BindGroupLayout defined in the PostProcessPipeline
            &BindGroupEntries::sequential((
                // Make sure to use the source view
                post_process_pipeline.source,
                // Use the sampler created for the pipeline
                &post_process_pipeline.sampler,
                // Set the settings binding
                settings_binding.clone(),
            )),
        );
        commands.insert_resource(GameOfLifeImageBindGroup(bind_group));

        Ok(())
    }
}

// Pipelines :

const SIZE: (u32, u32) = (1280, 720);
const WORKGROUP_SIZE: u32 = 8;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct VolumetricNodeLabel;

#[derive(Resource)]
struct MarchingCubesComputePipeline {
    texture_bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
}

impl FromWorld for MarchingCubesComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let texture_bind_group_layout = GameOfLifeImage::bind_group_layout(render_device);
        let shader = world.load_asset("shaders/game_of_life.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();

        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("init"),
        });
        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });

        MarchingCubesComputePipeline {
            texture_bind_group_layout,
            init_pipeline,
            update_pipeline,
        }
    }
}

enum GameOfLifeState {
    Loading,
    Init,
    Update,
}

struct VolumetricNode {
    state: GameOfLifeState,
}

impl Default for VolumetricNode {
    fn default() -> Self {
        Self {
            state: GameOfLifeState::Loading,
        }
    }
}

impl render_graph::Node for VolumetricNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<MarchingCubesComputePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            GameOfLifeState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline)
                {
                    self.state = GameOfLifeState::Init;
                }
            }
            GameOfLifeState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = GameOfLifeState::Update;
                }
            }
            GameOfLifeState::Update => {}
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
        let pipeline = world.resource::<MarchingCubesComputePipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, texture_bind_group, &[]);

        // select the pipeline based on the current state
        match self.state {
            GameOfLifeState::Loading => {}
            GameOfLifeState::Init => {
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.init_pipeline)
                    .unwrap();
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
            }
            GameOfLifeState::Update => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct PostProcessLabel;

#[derive(Resource)]
struct PostProcessPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

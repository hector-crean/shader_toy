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
use std::{borrow::Cow, marker::PhantomData, ops::Deref};

fn mesh_exp(mut meshes: ResMut<Assets<Mesh>>, mut mesh_handle: Handle<Mesh>) {
    let mesh = meshes.get(mesh_handle);

    if let Some(mesh) = mesh {}
}

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
        render_graph.add_node(VolumetricRenderingLabel, GameOfLifeNode::default());
        render_graph.add_node_edge(
            VolumetricRenderingLabel,
            bevy::render::graph::CameraDriverLabel,
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<VolumetricRenderingPipeline>();
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

    marching_cubes_mesh_handle: Handle<Mesh>,
}

#[derive(Clone, Resource)]
pub struct VolumetricRenderingPipeline {
    pub pipeline_id: CachedComputePipelineId,
    // pub shader: Handle<Shader>,
    pub marching_cubes_bind_group_layout: BindGroupLayout,
}

impl FromWorld for VolumetricRenderingPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        // We need to define the bind group layout used for our pipeline
        let marching_cubes_bind_group_layout = render_device.create_bind_group_layout(
            "volumetric_rendering_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                // The layout entries will only be visible in the fragment stage
                ShaderStages::COMPUTE,
                (
                    // The screen texture
                    texture_3d(TextureSampleType::Float { filterable: true }),
                    // The sampler that will be used to sample the screen texture
                    sampler(SamplerBindingType::Filtering),
                    // The settings uniform that will control the effect
                    uniform_buffer::<VolumetricRenderingSettings>(true),
                    storage_buffer(false),
                ),
            ),
        );

        // We can create the sampler here since it won't change at runtime and doesn't depend on the view
        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        // Get the shader handle
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/post_processing.wgsl");

        let pipeline_id = world
            .resource_mut::<PipelineCache>()
            // This will add the pipeline to the cache and queue it's creation
            .queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("volumetric_rendering_pipeline".into()),
                layout: vec![marching_cubes_bind_group_layout.clone()],
            });

        Self {
            pipeline_id,
            marching_cubes_bind_group_layout,
        }
    }
}

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
    volumetric_rendering_pipeline: Res<VolumetricRenderingPipeline>,
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
                &volumetric_rendering_pipeline.marching_cubes_bind_group_layout,
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

enum VolumetricRenderingState {
    Loading,
    Init,
    Update,
}

struct VolumetricRenderingNode {
    state: VolumetricRenderingState,
}

impl Default for VolumetricRenderingNode {
    fn default() -> Self {
        Self {
            state: VolumetricRenderingState::Loading,
        }
    }
}

impl render_graph::Node for VolumetricRenderingNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<VolumetricRenderingPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            VolumetricRenderingState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline)
                {
                    self.state = VolumetricRenderingState::Init;
                }
            }
            VolumetricRenderingState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = VolumetricRenderingState::Update;
                }
            }
            VolumetricRenderingState::Update => {}
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
        let pipeline = world.resource::<VolumetricRenderingPipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        //this is where we set the correct offset for the UniformIndexedBuffer...
        pass.set_bind_group(0, texture_bind_group, &[]);

        // select the pipeline based on the current state
        match self.state {
            VolumetricRenderingState::Loading => {}
            VolumetricRenderingState::Init => {
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.init_pipeline)
                    .unwrap();
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
            }
            VolumetricRenderingState::Update => {
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

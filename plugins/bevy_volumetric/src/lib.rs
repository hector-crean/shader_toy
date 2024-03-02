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

// WORK_CELLS = NUMTHREADS(x,y,z) * DISPATCH(x,y,z)

/*
 * Optimal Thread and Workgroup Configuration for Compute Shaders:
 *
 * When processing a 256-pixel texture, the goal is to maximize hardware efficiency by carefully selecting the dispatch size
 * and the number of threads per workgroup. This choice directly impacts how work is distributed across the GPU's processors,
 * often organized in 'warps' or equivalent units. A common warp size is 32 processors, but this can vary with the hardware.
 *
 * Efficient utilization of warps is crucial for performance. Ideally, we aim to fully occupy each warp with active threads while
 * minimizing the total number of warps required. This balance reduces idle time and maximizes parallel execution efficiency.
 *
 * Two configurations to process a 256-pixel texture illustrate different approaches:
 *
 * 1. High Dispatch Count, Single-thread Workgroups:
 *    - Dispatch Size: (16 x 16 x 1)
 *    - Threads per Workgroup (NUMTHREADS): (1 x 1 x 1)
 *    => Total: 256 work cells processed, with many small workgroups, potentially underutilizing warp capacity.
 *
 * 2. Single Dispatch, Multi-thread Workgroups:
 *    - Dispatch Size: (1 x 1 x 1)
 *    - Threads per Workgroup (NUMTHREADS): (16 x 16 x 1)
 *    => Total: 256 work cells processed, with a single workgroup fully utilizing warp capacity.
 *
 * Configuration Selection:
 * The optimal configuration depends on the specific hardware and the nature of the workload. For tasks like texture processing,
 * consider the texture's resolution (i.e., the number of pixels/texels in x, y, and z dimensions) to guide the tuning of
 * NUMTHREADS and DISPATCH parameters. The aim is to align these parameters with the GPU's architecture to minimize idle processors
 * and maximize parallelism.
 *
 * Formula:
 * NUMTHREADS(X_THREADS, Y_THREADS, Z_THREADS) * DISPATCH(RESOLUTION.x / X_THREADS, RESOLUTION.y / Y_THREADS, RESOLUTION.z / Z_THREADS)
 *
 * This formula ensures that the workload is evenly distributed across the available computational resources, taking full advantage
 * of the GPU's parallel processing capabilities. Adjusting the number of threads and the dispatch dimensions based on the specific
 * texture resolution and hardware characteristics can significantly impact performance.
 *
 * Note: It is essential to consider memory access patterns and synchronization requirements when defining NUMTHREADS and DISPATCH,
 * as these factors can also influence overall efficiency and performance.
 *
 * For WebGPU particular, constraints are placed on the
 */

use std::cmp::min;

struct GpuHardware {
    max_workgroup_size_x: u32,
    max_workgroup_size_y: u32,
    max_workgroup_size_z: u32,
    max_threads_per_workgroup: u32,
    max_workgroups_per_dimension: u32,
}

type WorkGroupSize = UVec3;
type WorkgroupDispatch = UVec3;

impl Default for GpuHardware {
    fn default() -> Self {
        Self {
            max_workgroup_size_x: 256,
            max_workgroup_size_y: 256,
            max_workgroup_size_z: 64,
            max_threads_per_workgroup: u32::MAX,
            max_workgroups_per_dimension: 65535,
        }
    }
}
impl GpuHardware {
    fn calculate_workgroup_configuration(
        &self,
        resolution: UVec3,
    ) -> (WorkGroupSize, WorkgroupDispatch) {
        // Calculate optimal NUMTHREADS within WebGPU limits and workload dimensions
        let x_threads = min(resolution.x, self.max_workgroup_size_x);
        let y_threads = min(resolution.y, self.max_workgroup_size_y);
        let z_threads = min(resolution.z, self.max_workgroup_size_z);

        // Compute DISPATCH dimensions to cover all data, respecting the maximum number of workgroups per dimension
        let dispatch_x = (resolution.x as f32 / x_threads as f32)
            .ceil()
            .min(self.max_workgroups_per_dimension as f32) as u32;
        let dispatch_y = (resolution.y as f32 / y_threads as f32)
            .ceil()
            .min(self.max_workgroups_per_dimension as f32) as u32;
        let dispatch_z = (resolution.z as f32 / z_threads as f32)
            .ceil()
            .min(self.max_workgroups_per_dimension as f32) as u32;

        // Return NUMTHREADS and DISPATCH configurations
        (
            UVec3::new(x_threads, y_threads, z_threads),
            UVec3::new(dispatch_x, dispatch_y, dispatch_z),
        )
    }
}

// Function to calculate NUMTHREADS and DISPATCH values

pub struct VolumetricRenderingPlugin {
    gpu_hardware: GpuHardware,
}

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
pub struct VolumetricRenderingBundle {
    // This will likely be an MRI scan, or other volumetric data
    vector_field_3d: Handle<Image>,
    // The isosurface etc.
    settings: VolumetricRenderingSettings,
    // We compute the 'marching cubes' buffer
    marching_cubes_grid_handle: Handle<Grid>,
    // marching_cubes_mesh_handle: Handle<Mesh>,
}

use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{
    BufferDescriptor, CachedRenderPipelineId, DynamicBindGroupEntries, PipelineCache,
    SpecializedRenderPipelines,
};
use bevy::render::renderer::RenderQueue;
use bevy::render::view::VisibleEntities;
use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    prelude::*,
    render::{
        render_asset::RenderAsset,
        render_resource::{BindGroup, Buffer, BufferInitDescriptor, BufferUsages, ShaderType},
        renderer::RenderDevice,
        Extract,
    },
};

use crate::loader::las_loader::PointCloudAsset;

#[derive(Component, Clone)]
pub struct PotreePointCloud {
    pub mesh: Handle<PointCloudAsset>,
    pub point_size: f32,
}

#[derive(Component, Clone, ShaderType)]
pub struct PointcloudPoint {
    pub transform: Mat4,
    pub point_size: f32,
}

impl PotreePointCloud {
    pub fn extract(
        mut commands: Commands,
        mut previous_len: Local<usize>,
        query: Extract<Query<(Entity, &PotreePointCloud, &GlobalTransform)>>,
    ) {
        let mut values = Vec::with_capacity(*previous_len);

        for (entity, point_cloud, transform) in query.iter() {
            values.push((
                entity,
                (
                    PointcloudPoint {
                        transform: transform.compute_matrix(),
                        point_size: point_cloud.point_size,
                    },
                    point_cloud.mesh.clone(),
                ),
            ));
        }
        *previous_len = values.len();
        commands.insert_or_spawn_batch(values);
    }
}

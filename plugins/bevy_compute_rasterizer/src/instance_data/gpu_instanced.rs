use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query, Res},
    },
    render::{
        render_resource::{Buffer, BufferInitDescriptor, BufferUsages},
        renderer::RenderDevice,
    },
};

use super::cpu_instanced::CpuInstancesData;

#[derive(Component)]
pub struct GpuInstancesData {
    pub buffer: Buffer,
    pub length: usize,
}

impl GpuInstancesData {
    pub fn prepare(
        mut commands: Commands,
        query: Query<(Entity, &CpuInstancesData)>,
        render_device: Res<RenderDevice>,
    ) {
        for (entity, instance_data) in &query {
            let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("instance data buffer"),
                contents: bytemuck::cast_slice(instance_data.as_slice()),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });
            commands.entity(entity).insert(GpuInstancesData {
                buffer,
                length: instance_data.len(),
            });
        }
    }
}

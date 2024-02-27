use std::{hash::Hash, marker::PhantomData};

use bevy::{
    core_pipeline::core_3d::Transparent3d,
    ecs::{
        query::QueryItem,
        system::{lifetimeless::*, SystemParamItem},
    },
    pbr::{
        MeshPipeline, MeshPipelineKey, RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup,
    },
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::{GpuBufferInfo, MeshVertexBufferLayout},
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
            RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::RenderDevice,
        view::{ExtractedView, NoFrustumCulling},
        Render, RenderApp, RenderSet,
    },
};

use crate::{
    render::{
        render_pipeline::{queue_instanced_material, DrawPointcloud, PointcloudRenderPipeline},
        shaders::load_instancing_shaders,
    },
    uniforms::{cpu_instanced::InstancesData, gpu_instanced::GpuInstancesData},
    util::InternalShaders,
};

#[derive(Default)]
pub struct InstancedMaterialPlugin<M: Material> {
    pub _marker: PhantomData<M>,
}

impl<M: Material> Plugin for InstancedMaterialPlugin<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<InstancesData>::default())
            .init_resource::<InternalShaders>();
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawPointcloud<M>>()
            .init_resource::<SpecializedMeshPipelines<PointcloudRenderPipeline<M>>>()
            .add_systems(
                Render,
                (
                    queue_instanced_material::<M>.in_set(RenderSet::QueueMeshes),
                    GpuInstancesData::prepare.in_set(RenderSet::PrepareResources),
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        load_instancing_shaders(app);

        app.sub_app_mut(RenderApp)
            .init_resource::<PointcloudRenderPipeline<M>>();
    }
}

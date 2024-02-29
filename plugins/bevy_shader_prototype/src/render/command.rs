use bevy::{
    ecs::{
        query::ROQueryItem,
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_phase::{
            PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::BindGroup,
        view::ViewUniformOffset,
    },
    render::{
        render_resource::*,
        renderer::RenderDevice,
        view::{ExtractedView, ViewUniforms},
    },
    utils::HashMap,
};

// pub type DrawSdfCommand<T> = (SetItemPipeline,);

#[derive(Component, Debug)]
pub struct ShapeViewBindGroup {
    value: BindGroup,
}

pub fn prepare_shape_view_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    marching_cubes_pipeline: Res<ShapePipelines>,
    view_uniforms: Res<ViewUniforms>,
    views: Query<Entity, With<ExtractedView>>,
) {
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        for entity in views.iter() {
            let view_bind_group = render_device.create_bind_group(
                "shape_view_bind_group",
                &shape_pipeline.view_layout,
                &BindGroupEntries::single(view_binding.clone()),
            );

            commands.entity(entity).insert(ShapeViewBindGroup {
                value: view_bind_group,
            });
        }
    }
}

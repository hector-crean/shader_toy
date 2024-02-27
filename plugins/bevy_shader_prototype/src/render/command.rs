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

pub type DrawSdfCommand<T> = (SetItemPipeline,);

#[derive(Component, Debug)]
struct ShapeViewBindGroup {
    value: BindGroup,
}

impl ShapeViewBindGroup {
    fn prepare() {}
}

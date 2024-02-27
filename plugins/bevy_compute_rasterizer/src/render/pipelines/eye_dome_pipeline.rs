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
use std::{hash::Hash, marker::PhantomData};

use crate::render::shaders::DEFAULT_EYE_DOME_PIPELINE_SHADER;

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct EyeDomeRenderPipelineFlags: u32 {
        const NONE               = 0;
    }
}

impl EyeDomeRenderPipelineFlags {
    pub fn shader_defs(&self) -> Vec<ShaderDefVal> {
        let mut shader_defs = Vec::new();

        shader_defs
    }
}

pub struct EyeDomeRenderPipelineKey<M: Material> {
    pub flags: EyeDomeRenderPipelineFlags,
    pub bind_group_data: M::Data,
}

impl<M: Material> Eq for EyeDomeRenderPipelineKey<M> where M::Data: PartialEq {}

impl<M: Material> PartialEq for EyeDomeRenderPipelineKey<M>
where
    M::Data: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.flags == other.flags && self.bind_group_data == other.bind_group_data
    }
}

impl<M: Material> Clone for EyeDomeRenderPipelineKey<M>
where
    M::Data: Clone,
{
    fn clone(&self) -> Self {
        Self {
            flags: self.flags,
            bind_group_data: self.bind_group_data.clone(),
        }
    }
}

impl<M: Material> Hash for EyeDomeRenderPipelineKey<M>
where
    M::Data: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.flags.hash(state);
        self.bind_group_data.hash(state);
    }
}

#[derive(Resource)]
pub struct EyeDomeRenderPipeline<M: Material> {
    shader: Handle<Shader>,
    eye_dome_image_layout: BindGroupLayout,
    multisampled_eye_dome_image_layout: BindGroupLayout,
    marker: PhantomData<M>,
}

impl<M: Material> FromWorld for EyeDomeRenderPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        let render_device = world.resource::<RenderDevice>();

        let shader = match M::vertex_shader() {
            ShaderRef::Default => asset_server.load(DEFAULT_EYE_DOME_PIPELINE_SHADER),
            ShaderRef::Handle(handle) => handle,
            ShaderRef::Path(path) => asset_server.load(path),
        };

        let eye_dome_image_layout = render_device.create_bind_group_layout(
            Some("EyeDomeImageLayout"),
            &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: false },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        );

        let multisampled_eye_dome_image_layout = render_device.create_bind_group_layout(
            Some("MultisampledEyeDomeImageLayout"),
            &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: false },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: true,
                },
                count: None,
            }],
        );

        EyeDomeRenderPipeline {
            shader,
            eye_dome_image_layout,
            multisampled_eye_dome_image_layout,
            marker: PhantomData,
        }
    }
}

impl<M: Material> SpecializedMeshPipeline for EyeDomeRenderPipeline<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    type Key = EyeDomeRenderPipelineKey<M>;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let msaa = key.msaa;

        let descriptor = RenderPipelineDescriptor {
            label: Some("EyeDomeLightingPipeline".into()),
            layout: vec![if msaa > 1 {
                self.multisampled_eye_dome_image_layout.clone()
            } else {
                self.eye_dome_image_layout.clone()
            }],
            vertex: VertexState {
                shader: DEFAULT_EYE_DOME_PIPELINE_SHADER,
                shader_defs: if msaa > 1 {
                    vec!["MULTISAMPLED".into()]
                } else {
                    default()
                },
                entry_point: "vertex".into(),
                buffers: vec![VertexBufferLayout {
                    array_stride: 8,
                    step_mode: VertexStepMode::Vertex,
                    attributes: vec![VertexAttribute {
                        format: VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: msaa,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                shader: DEFAULT_EYE_DOME_PIPELINE_SHADER,
                shader_defs: if msaa > 1 {
                    vec!["MULTISAMPLED".into()]
                } else {
                    default()
                },
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::Zero,
                            dst_factor: BlendFactor::SrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent {
                            src_factor: BlendFactor::One,
                            dst_factor: BlendFactor::Zero,
                            operation: BlendOperation::Add,
                        },
                    }),
                    write_mask: ColorWrites::COLOR,
                })],
            }),
            push_constant_ranges: vec![PushConstantRange {
                stages: ShaderStages::FRAGMENT,
                range: 0..std::mem::size_of::<f32>() as u32,
            }],
        };

        Ok(descriptor)
    }
}

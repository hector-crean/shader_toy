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

use crate::render::shaders::DEFAULT_POINTCLOUD_PIPELINE_SHADER;

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct PointcloudRenderPipelineFlags: u32 {
        const NONE               = 0;
        const SPHERICAL          = (1 <<  0);
        const CUBOID          = (1 <<  1);
    }
}

impl PointcloudRenderPipelineFlags {
    pub fn shader_defs(&self) -> Vec<ShaderDefVal> {
        let mut shader_defs = Vec::new();

        if (self.bits() & PointcloudRenderPipelineFlags::SPHERICAL.bits()) != 0 {
            shader_defs.push("SPHERICAL".into());
        }
        if (self.bits() & PointcloudRenderPipelineFlags::CUBOID.bits()) != 0 {
            shader_defs.push("CUBOID".into());
        }

        shader_defs
    }
}

pub struct PointcloudRenderPipelineKey<M: Material> {
    pub flags: PointcloudRenderPipelineFlags,
    pub bind_group_data: M::Data,

    pub colored: bool,
    pub animated: bool,
    pub msaa: u32,
}

impl<M: Material> Eq for PointcloudRenderPipelineKey<M> where M::Data: PartialEq {}

impl<M: Material> PartialEq for PointcloudRenderPipelineKey<M>
where
    M::Data: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.flags == other.flags && self.bind_group_data == other.bind_group_data
    }
}

impl<M: Material> Clone for PointcloudRenderPipelineKey<M>
where
    M::Data: Clone,
{
    fn clone(&self) -> Self {
        Self {
            flags: self.flags,
            bind_group_data: self.bind_group_data.clone(),
            colored: self.colored.clone,
            animated: self.animated.clone,
            msaa: self.msaa.clone(),
        }
    }
}

impl<M: Material> Hash for PointcloudRenderPipelineKey<M>
where
    M::Data: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.flags.hash(state);
        self.bind_group_data.hash(state);
    }
}

#[derive(Resource)]
pub struct PointcloudRenderPipeline<M: Material> {
    shader: Handle<Shader>,
    pub view_layout: BindGroupLayout,
    pub entity_layout: BindGroupLayout,
    pub animated_entity_layout: BindGroupLayout,
    pub model_layout: BindGroupLayout,

    pub instanced_point_quad: Buffer,
    marker: PhantomData<M>,
}

const QUAD_VERTEX_BUF: &[f32] = &[0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0];

impl<M: Material> FromWorld for PointcloudRenderPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        let shader = match M::vertex_shader() {
            ShaderRef::Default => asset_server.load(DEFAULT_POINTCLOUD_PIPELINE_SHADER),
            ShaderRef::Handle(handle) => handle,
            ShaderRef::Path(path) => asset_server.load(path),
        };

        let render_device = world.resource::<RenderDevice>();

        let instanced_point_quad = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: "instanced point quad".into(),
            contents: bytemuck::cast_slice(QUAD_VERTEX_BUF),
            usage: BufferUsages::VERTEX,
        });

        let view_layout = render_device.create_bind_group_layout(
            Some("PointCloudViewLabel"),
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        );

        let entity_layout = render_device.create_bind_group_layout(
            Some("PointCloudViewLayout"),
            &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        );

        let animated_entity_layout = render_device.create_bind_group_layout(
            Some("PointCloudViewLayout"),
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        );

        let model_layout = render_device.create_bind_group_layout(
            Some("PointCloudModelLayout"),
            &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: None,
                },
                count: None,
            }],
        );

        PointcloudRenderPipeline {
            shader,
            marker: PhantomData,
            view_layout,
            model_layout,
            entity_layout,
            animated_entity_layout,
            instanced_point_quad,
        }
    }
}

impl<M: Material> SpecializedMeshPipeline for PointcloudRenderPipeline<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    type Key = PointcloudRenderPipelineKey<M>;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let PointcloudRenderPipelineKey {
            colored,
            animated,
            msaa,
            ..
        } = key;

        let descriptor = RenderPipelineDescriptor {
            label: Some("point_cloud_pipeline".into()),
            layout: vec![
                self.view_layout.clone(),
                if animated {
                    self.animated_entity_layout.clone()
                } else {
                    self.entity_layout.clone()
                },
                self.model_layout.clone(),
            ],
            vertex: VertexState {
                shader: DEFAULT_POINTCLOUD_PIPELINE_SHADER,
                shader_defs: {
                    let mut defs = Vec::new();
                    if colored {
                        defs.push("COLORED".into());
                    }
                    if animated {
                        defs.push("ANIMATED".into());
                    }
                    defs
                },
                entry_point: "main".into(),
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
            fragment: Some(FragmentState {
                shader: DEFAULT_POINTCLOUD_PIPELINE_SHADER,
                shader_defs: {
                    let mut defs = Vec::new();
                    if colored {
                        defs.push("COLORED".into());
                    }
                    if animated {
                        defs.push("ANIMATED".into());
                    }
                    defs
                },
                entry_point: "main".into(),
                targets: vec![
                    Some(ColorTargetState {
                        format: TextureFormat::Rgba8UnormSrgb,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    }),
                    Some(ColorTargetState {
                        format: TextureFormat::R32Float,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::RED,
                    }),
                ],
            }),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Greater,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: MultisampleState {
                count: msaa,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            push_constant_ranges: default(),
        };

        Ok(descriptor)
    }
}

use bevy::{
    core::{Pod, Zeroable},
    ecs::{component::Component, query::QueryItem},
    math::{Vec3, Vec4},
    prelude::Deref,
    render::extract_component::ExtractComponent,
};

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct CpuInstance {
    position: Vec3,
    scale: f32,
    color: [f32; 4],
}

impl CpuInstance {
    pub fn new(position: Vec3, scale: f32, color: [f32; 4]) -> Self {
        Self {
            position,
            scale,
            color,
        }
    }
}

#[derive(Component, Deref)]
pub struct CpuInstancesData {
    pub data: Vec<CpuInstance>,
}

impl CpuInstancesData {
    pub fn new(data: Vec<CpuInstance>) -> Self {
        Self { data }
    }
}

impl ExtractComponent for CpuInstancesData {
    type QueryData = &'static CpuInstancesData;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::QueryData>) -> Option<Self> {
        Some(CpuInstancesData {
            data: item.data.clone(),
        })
    }
}

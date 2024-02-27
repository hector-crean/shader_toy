use bevy::{
    core::{Pod, Zeroable},
    ecs::{component::Component, query::QueryItem},
    math::Vec3,
    prelude::Deref,
    render::extract_component::ExtractComponent,
};

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Instance {
    position: Vec3,
    scale: f32,
    color: [f32; 4],
}

impl Instance {
    pub fn new(position: Vec3, scale: f32, color: [f32; 4]) -> Self {
        Self {
            position,
            scale,
            color,
        }
    }
}

#[derive(Component, Deref)]
pub struct InstancesData {
    pub data: Vec<Instance>,
}

impl InstancesData {
    pub fn new(data: Vec<Instance>) -> Self {
        Self { data }
    }
}

impl ExtractComponent for InstancesData {
    type QueryData = &'static InstancesData;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::QueryData>) -> Option<Self> {
        Some(InstancesData {
            data: item.data.clone(),
        })
    }
}

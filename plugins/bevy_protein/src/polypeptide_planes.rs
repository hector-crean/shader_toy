use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::{
    app::{Plugin, Startup, Update},
    asset::{AssetApp, AssetServer, Assets, Handle},
    ecs::{
        change_detection::DetectChanges,
        system::{Command, Commands, In, Res, ResMut, Resource},
    },
    math::{
        cubic_splines::{CubicBSpline, CubicBezier, CubicGenerator, *},
        primitives::Plane3d,
        Vec3,
    },
    pbr::{MaterialMeshBundle, StandardMaterial},
    render::{color::Color, mesh::Mesh},
    transform::components::Transform,
    utils::default,
};
use bevy_geometry::TangentSpace;

use crate::polypeptide_plane::PolypeptidePlane;

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
pub struct PolypeptidePlanes(Vec<PolypeptidePlane>);

impl From<Vec<PolypeptidePlane>> for PolypeptidePlanes {
    fn from(value: Vec<PolypeptidePlane>) -> Self {
        Self(value)
    }
}

impl PolypeptidePlanes {
    pub fn new(planes: Vec<PolypeptidePlane>) -> Self {
        Self(planes)
    }
    pub fn discrete_tangent_spaces(&self) -> Vec<TangentSpace> {
        self.0.iter().map(|plane| plane.tangent_space).collect()
    }
}

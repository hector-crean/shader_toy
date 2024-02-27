use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::MeshVertexAttribute,
        render_resource::{PrimitiveTopology, VertexFormat},
    },
    utils::thiserror::{self, Error},
};
use las::Read;
use opd_parser::Frames;

#[repr(transparent)]
pub struct Point {
    pub inner: [f32; 3],
}
impl From<[f32; 3]> for Point {
    fn from(inner: [f32; 3]) -> Self {
        Self { inner }
    }
}
impl Point {
    pub fn min(&self, other: &Self) -> Self {
        Point {
            inner: [
                self.inner[0].min(other.inner[0]),
                self.inner[1].min(other.inner[1]),
                self.inner[2].min(other.inner[2]),
            ],
        }
    }
    pub fn max(&self, other: &Self) -> Self {
        Point {
            inner: [
                self.inner[0].max(other.inner[0]),
                self.inner[1].max(other.inner[1]),
                self.inner[2].max(other.inner[2]),
            ],
        }
    }
}

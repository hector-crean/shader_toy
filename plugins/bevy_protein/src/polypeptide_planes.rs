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

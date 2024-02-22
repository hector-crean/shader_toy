use bevy::math::{cubic_splines::*, vec2, Vec3};

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct TangentSpace {
    pub position: Vec3,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub bitangent: Vec3,
}

impl TangentSpace {
    pub fn new(position: Vec3, normal: Vec3, tangent: Vec3, bitangent: Vec3) -> Self {
        Self {
            position,
            normal,
            tangent,
            bitangent,
        }
    }
}

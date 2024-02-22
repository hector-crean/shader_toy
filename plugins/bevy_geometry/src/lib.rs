use bevy::math::Vec3;

pub mod primitives;

// Bevy already has a number of geo primitives, which we can use.

// Molstar has a number of primitives it uses, and also defines some more complex composite geometries

// Perhaps use a CAD Kernel to create some more complex composites of geometries? i.e. rust-trunk
// https://ricos.gitlab.io/truck-tutorial/v0.1/overview.html

//continuous curve through a manifold is often referred to as a path or geodesic
//At each point on the manifold one can define a tangent space (which is effectively the local coordinate basis)

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct TangentSpace {
    pub position: Vec3,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub binormal: Vec3,
}

impl TangentSpace {
    pub fn new(position: Vec3, normal: Vec3, tangent: Vec3, binormal: Vec3) -> Self {
        Self {
            position,
            normal,
            tangent,
            binormal,
        }
    }
}

// the parameter t is the length along the geodesic

pub trait ContinousGeodesic {
    fn position_fn(&self, t: f32) -> Vec3;
    fn normal_fn(&self, t: f32) -> Vec3;
    fn tangent_fn(&self, t: f32) -> Vec3;
    fn binormal_fn(&self, t: f32) -> Vec3;
}

pub struct F32Range {
    start: f32,
    end: f32,
    step: f32,
    next: f32,
}

impl F32Range {
    pub fn new(start: f32, end: f32, step: f32) -> F32Range {
        F32Range {
            start,
            end,
            step,
            next: start,
        }
    }
}

impl Iterator for F32Range {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.end {
            return None;
        }
        let current = self.next;
        self.next += self.step;
        Some(current)
    }
}

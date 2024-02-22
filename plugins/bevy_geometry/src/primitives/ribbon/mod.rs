use crate::{ContinousGeodesic, F32Range, TangentSpace};
use bevy::log::{self, info};
use bevy::prelude::CubicGenerator;
use bevy::{
    math::{
        cubic_splines::{CubicBSpline, CubicCurve},
        primitives::Primitive3d,
        Vec3,
    },
    render::{
        mesh::{Indices, Mesh, Meshable, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};

/// A Ribbon primitive
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Ribbon {
    width: f32,
    thickness: f32,
    discrete_geodesic: Vec<TangentSpace>,
    segments: u32,
}

impl Primitive3d for Ribbon {}

impl Default for Ribbon {
    /// Returns the default [`Ribbon`] with a radius of `0.5`.
    fn default() -> Self {
        Self {
            discrete_geodesic: vec![],
            width: 1.,
            thickness: 0.5,
            segments: 32,
        }
    }
}

impl Ribbon {
    /// Create a new [`Ribbon`] from a `radius`
    pub const fn new(
        discrete_geodesic: Vec<TangentSpace>,
        width: f32,
        thickness: f32,
        segments: u32,
    ) -> Self {
        Self {
            discrete_geodesic,
            width,
            thickness,
            segments,
        }
    }

    fn position_interpolator(&self) -> CubicCurve<Vec3> {
        let points: Vec<Vec3> = self
            .discrete_geodesic
            .iter()
            .map(|space| space.position)
            .collect();

        CubicBSpline::new(points).to_curve()
    }
    fn normal_interpolator(&self) -> CubicCurve<Vec3> {
        let points: Vec<Vec3> = self
            .discrete_geodesic
            .iter()
            .map(|space| space.normal)
            .collect();

        CubicBSpline::new(points).to_curve()
    }
    fn binormal_interpolator(&self) -> CubicCurve<Vec3> {
        let points: Vec<Vec3> = self
            .discrete_geodesic
            .iter()
            .map(|space| space.binormal)
            .collect();

        CubicBSpline::new(points).to_curve()
    }
    fn tangent_interpolator(&self) -> CubicCurve<Vec3> {
        let points: Vec<Vec3> = self
            .discrete_geodesic
            .iter()
            .map(|space| space.tangent)
            .collect();

        CubicBSpline::new(points).to_curve()
    }
}

/////

/// A builder used for creating a [`Mesh`] with a [`Ribbon`] shape.
#[derive(Clone, Debug)]
pub struct RibbonMeshBuilder {
    /// The [`Ribbon`] shape.
    pub ribbon: Ribbon,
    pub segments: u32,
    pub position_interpolator: CubicCurve<Vec3>,
    pub tangent_interpolator: CubicCurve<Vec3>,
    pub normal_interpolator: CubicCurve<Vec3>,
    pub binormal_interpolator: CubicCurve<Vec3>,
}

impl ContinousGeodesic for RibbonMeshBuilder {
    fn position_fn(&self, t: f32) -> Vec3 {
        self.position_interpolator.position(t)
    }
    fn binormal_fn(&self, t: f32) -> Vec3 {
        self.binormal_interpolator.position(t)
    }
    fn normal_fn(&self, t: f32) -> Vec3 {
        self.normal_interpolator.position(t)
    }
    fn tangent_fn(&self, t: f32) -> Vec3 {
        self.tangent_interpolator.position(t)
    }
}

// impl Default for RibbonMeshBuilder {
//     fn default() -> Self {
//         let default_interpolator = CubicBSpline::new([Vec3::NEG_ONE, Vec3::ONE]).to_curve();

//         Self {
//             ribbon: Ribbon::default(),
//             segments: 1,
//             position_interpolator: default_interpolator.clone(),
//             tangent_interpolator: default_interpolator.clone(),
//             normal_interpolator: default_interpolator.clone(),
//             binormal_interpolator: default_interpolator.clone(),
//         }
//     }
// }

impl RibbonMeshBuilder {
    /// Creates a new [`RibbonMeshBuilder`] from the given radius, a height,
    /// and a resolution used for the top and bottom.
    #[inline]
    pub fn new(
        discrete_geodesic: Vec<TangentSpace>,
        width: f32,
        thickness: f32,
        segments: u32,
    ) -> Self {
        let ribbon = Ribbon::new(discrete_geodesic, width, thickness, segments);
        let position_interpolator = ribbon.position_interpolator();
        let tangent_interpolator = ribbon.tangent_interpolator();
        let normal_interpolator = ribbon.normal_interpolator();
        let binormal_interpolator = ribbon.binormal_interpolator();

        Self {
            ribbon,
            segments,
            position_interpolator,
            tangent_interpolator,
            normal_interpolator,
            binormal_interpolator,
        }
    }

    /// Sets the number of segments along the height of the Ribbon.
    /// Must be greater than `0` for geometry to be generated.
    #[inline]
    pub const fn segments(mut self, segments: u32) -> Self {
        self.segments = segments;
        self
    }

    /// Builds a [`Mesh`] based on the configuration in `self`.
    pub fn build(&self) -> Mesh {
        // parametric value t representing length along geodesic. Here t: [0, number_control_points - 3],
        // where t can be a f32

        let T = self.ribbon.discrete_geodesic.len() as f32 - 3.;
        let geodesic_length = T.clamp(0., f32::MAX);

        let mut positions = Vec::<Vec3>::new();
        let mut normals = Vec::<Vec3>::new();
        let mut uvs = Vec::<Vec3>::new();
        let mut indices = Vec::<u32>::new();

        let dt = geodesic_length / (self.segments as f32);

        for (idx, t) in F32Range::new(0., geodesic_length, dt).enumerate() {
            let idx = idx as u32;

            info!("idx: {:?}, t: {:?}", idx, t);

            let p_1 = self.position_fn(t);
            let n_1 = self.normal_fn(t);
            let bt_1 = self.binormal_fn(t);

            let p_2 = self.position_fn(t + dt);
            let n_2 = self.normal_fn(t + dt);
            let bt_2 = self.binormal_fn(t + dt);

            let vertex_11 = p_1 + 0.5 * self.ribbon.width * bt_1;
            let vertex_12 = p_1 - 0.5 * self.ribbon.width * bt_1;

            let vertex_21 = p_2 + 0.5 * self.ribbon.width * bt_2;
            let vertex_22 = p_2 - 0.5 * self.ribbon.width * bt_2;

            //Vertices
            positions.push(vertex_11);
            positions.push(vertex_12);
            positions.push(vertex_21);
            positions.push(vertex_22);

            normals.push(n_1);
            normals.push(n_1);
            normals.push(n_2);
            normals.push(n_2);

            indices.push(idx);
            indices.push(idx + 3);
            indices.push(idx + 1);

            indices.push(idx);
            indices.push(idx + 2);
            indices.push(idx + 3);
        }

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_indices(Indices::U32(indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        // .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    }
}

impl Meshable for Ribbon {
    type Output = RibbonMeshBuilder;

    fn mesh(&self) -> Self::Output {
        RibbonMeshBuilder::new(
            self.discrete_geodesic.clone(),
            self.width,
            self.thickness,
            self.segments,
        )
    }
}

impl From<Ribbon> for Mesh {
    fn from(ribbon: Ribbon) -> Self {
        ribbon.mesh().build()
    }
}

impl From<RibbonMeshBuilder> for Mesh {
    fn from(ribbon: RibbonMeshBuilder) -> Self {
        ribbon.build()
    }
}

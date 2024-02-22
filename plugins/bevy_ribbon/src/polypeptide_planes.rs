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

use crate::{polypeptide_plane::PolypeptidePlane, tangent_space::TangentSpace};


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
    fn discrete_tangent_spaces(&self) -> Vec<TangentSpace> {
        self.0.iter().map(|plane| plane.tangent_space).collect()
    }

    fn positions_spline(&self) -> CubicCurve<Vec3> {
        let points: Vec<Vec3> = self
            .discrete_tangent_spaces()
            .iter()
            .map(|space| space.position)
            .collect();

        CubicBSpline::new(points).to_curve()
    }
    fn normals_spline(&self) -> CubicCurve<Vec3> {
        let points: Vec<Vec3> = self
            .discrete_tangent_spaces()
            .iter()
            .map(|space| space.normal)
            .collect();

        CubicBSpline::new(points).to_curve()
    }
    fn bitangents_spline(&self) -> CubicCurve<Vec3> {
        let points: Vec<Vec3> = self
            .discrete_tangent_spaces()
            .iter()
            .map(|space| space.bitangent)
            .collect();

        CubicBSpline::new(points).to_curve()
    }
    fn tangents_spline(&self) -> CubicCurve<Vec3> {
        let points: Vec<Vec3> = self
            .discrete_tangent_spaces()
            .iter()
            .map(|space| space.tangent)
            .collect();

        CubicBSpline::new(points).to_curve()
    }

    

    #[rustfmt::skip]
    pub fn create_ribbon_mesh(&self, width: f32,) -> Mesh {


        let normals_fn = self.normals_spline();
        let bitangents_fn = self.bitangents_spline();
        let tangents_fn = self.tangents_spline();
        let positions_fn = self.positions_spline();

        let mut vertices = Vec::<Vec3>::new();
        let mut normals = Vec::<Vec3>::new();
        let mut indices = Vec::<u32>::new();


        for t in 0..=(self.0.len() - 3) {


            let p_1 = positions_fn.position(t as f32);
            let n_1 = normals_fn.position(t as f32);
            let bt_1 = bitangents_fn.position(t as f32);

            let p_2 = positions_fn.position(t as f32 + 1.);
            let n_2 = normals_fn.position(t as f32 + 1.);
            let bt_2 = bitangents_fn.position(t as f32 + 1.);


            let vertex_11 = p_1 + 0.5 * width * bt_1;
            let vertex_12 = p_1 - 0.5 * width * bt_1;

            let vertex_21 = p_2 + 0.5 * width * bt_2;
            let vertex_22 = p_2 - 0.5 * width * bt_2;

            //Vertices
            vertices.push(vertex_11);
            vertices.push(vertex_12);
            vertices.push(vertex_21);
            vertices.push(vertex_22);

            normals.push(n_1);
            normals.push(n_1);
            normals.push(n_2);
            normals.push(n_2);


            indices.push(t as u32);
            indices.push(t as u32 + 3);
            indices.push(t as u32 + 1);

            indices.push(t as u32);
            indices.push(t as u32 + 2);
            indices.push(t as u32 + 3);

        }

       


        // Keep the mesh data accessible in future frames to be able to mutate it in toggle_texture.
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            // Each array is an [x, y, z] coordinate in local space.
            // Meshes always rotate around their local [0, 0, 0] when a rotation is applied to their Transform.
            // By centering our mesh around the origin, rotating the mesh preserves its center of mass.
            vertices
        )
       
      
        // For meshes with flat shading, normals are orthogonal (pointing out) from the direction of
        // the surface.
        // Normals are required for correct lighting calculations.
        // Each array represents a normalized vector, which length should be equal to 1.0.
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            normals,
        )
        // Create the triangles out of the 24 vertices we created.
        // To construct a square, we need 2 triangles, therefore 12 triangles in total.
        // To construct a triangle, we need the indices of its 3 defined vertices, adding them one
        // by one, in a counter-clockwise order (relative to the position of the viewer, the order
        // should appear counter-clockwise from the front of the triangle, in this case from outside the cube).
        // Read more about how to correctly build a mesh manually in the Bevy documentation of a Mesh,
        // further examples and the implementation of the built-in shapes.
        .with_inserted_indices(Indices::U32(indices))
}
}

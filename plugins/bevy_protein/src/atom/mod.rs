use bevy::prelude::*;
use bevy::{ecs::component::Component, math::primitives::Sphere};
use periodic_table_on_an_enum::Element;

// Pdb file coordinates are typically expressed in Ångstroms (Å).
// The size of an atom typically ranges from about  0.5 Ångstroms (Å)
// to about 2.5 Å.

#[derive(Component)]
pub struct Atom {
    pub atomic_data: pdbtbx::Atom,
}

impl Atom {
    pub fn new(atomic_data: &pdbtbx::Atom) -> Self {
        Self {
            atomic_data: atomic_data.clone(),
        }
    }
}

impl Atom {
    pub fn spawn(
        self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) {
        let (x, y, z) = self.atomic_data.pos();
        let sphere_mesh = Sphere::new(0.25);
        // Render the mesh with the custom texture using a PbrBundle, add the marker.
        commands.spawn((PbrBundle {
            mesh: meshes.add(sphere_mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::BLUE,
                ..default()
            }),
            transform: Transform::from_xyz(x as f32, y as f32, z as f32),
            ..default()
        },));
    }
}

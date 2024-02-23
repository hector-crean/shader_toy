pub mod atom;
pub mod bonds;
pub mod polypeptide;
pub mod protein_asset_loader;

use polypeptide::{polypeptide_plane, polypeptide_planes};

use std::ops::Range;

use bevy::{
    app::{Plugin, Update},
    asset::{AssetApp, AssetEvent, Assets},
    ecs::{
        event::EventReader,
        system::{Commands, ResMut},
    },
    log::info,
    math::{primitives::Sphere, Vec3},
    pbr::{PbrBundle, StandardMaterial},
    prelude::SpatialBundle,
    render::{color::Color, mesh::Mesh, view::NoFrustumCulling},
    utils::default,
};
use bevy_geometry::primitives::ribbon::Ribbon;

use crate::atom::Atom;
use bevy_instanced::{
    instance_data::cpu_instanced::{CpuInstance, CpuInstancesData},
    plugin::InstancedMaterialPlugin,
};
use protein_asset_loader::{ProteinAsset, ProteinAssetLoader};

pub struct ProteinPlugin;

impl Plugin for ProteinPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(InstancedMaterialPlugin::<StandardMaterial>::default())
            .init_asset::<ProteinAsset>()
            .register_asset_loader(ProteinAssetLoader)
            // .register_asset_processor::<LoadTransformAndSave<CifAssetLoader, CifAssetTransformer, ProteinAssetSaver>>(
            //     LoadTransformAndSave::new(CifAssetTransformer, ProteinAssetSaver),
            // )
            // .set_default_asset_processor::<LoadTransformAndSave<CifAssetLoader, CifAssetTransformer, ProteinAssetSaver>>("cif")
            .add_systems(Update, Self::setup_protein);
    }
}

impl ProteinPlugin {
    fn setup_protein(
        mut commands: Commands,
        mut ev_protein_asset: EventReader<AssetEvent<ProteinAsset>>,
        protein_assets: ResMut<Assets<ProteinAsset>>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        for ev in ev_protein_asset.read() {
            match ev {
                AssetEvent::Added { id } => {
                    // a texture was just loaded or changed!

                    info!("protein asset loaded: {:?}", id);

                    let protein_asset = protein_assets.get(*id);

                    match protein_asset {
                        Some(ProteinAsset {
                            polypeptide_planes,
                            pdb,
                        }) => {
                            // for atom in pdb.atoms() {
                            //     Atom::new(atom).spawn(&mut commands, &mut meshes, &mut materials);
                            // }

                            commands.spawn((
                                meshes.add(Sphere::new(0.5)),
                                SpatialBundle::INHERITED_IDENTITY,
                                CpuInstancesData::new(
                                    pdb.atoms()
                                        .map(|atom| {
                                            let (x, y, z) = atom.pos();
                                            CpuInstance::new(
                                                Vec3::new(x as f32, y as f32, z as f32),
                                                1.0,
                                                Color::BLUE.as_rgba_f32(),
                                            )
                                        })
                                        .collect(),
                                ),
                                // NOTE: Frustum culling is done based on the Aabb of the Mesh and the GlobalTransform.
                                // As the cube is at the origin, if its Aabb moves outside the view frustum, all the
                                // instanced cubes will be culled.
                                // The InstanceMaterialData contains the 'GlobalTransform' information for this custom
                                // instancing, and that is not taken into account with the built-in frustum culling.
                                // We must disable the built-in frustum culling by adding the `NoFrustumCulling` marker
                                // component to avoid incorrect culling.
                                NoFrustumCulling,
                            ));

                            let discrete_geodesic = polypeptide_planes
                                .0
                                .iter()
                                .map(|plane| plane.tangent_space)
                                .collect::<Vec<_>>();

                            let t_domain = Range {
                                start: 0,
                                end: discrete_geodesic.len(),
                            };

                            let ribbon = Ribbon::new(
                                &discrete_geodesic,
                                t_domain,
                                5.,
                                1.,
                                discrete_geodesic.len() as u32 * 10 as u32,
                            );

                            let ribbon_mesh_handle = meshes.add(ribbon);

                            // Render the mesh with the custom texture using a PbrBundle, add the marker.
                            commands.spawn((PbrBundle {
                                mesh: ribbon_mesh_handle,
                                material: materials.add(StandardMaterial {
                                    base_color: Color::RED,
                                    ..default()
                                }),
                                ..default()
                            },));
                        }
                        None => {}
                    }
                }
                AssetEvent::Modified { id } => {
                    // an image was modified
                }
                AssetEvent::Removed { id } => {
                    // an image was unloaded
                }
                _ => {}
            }
        }
    }
}

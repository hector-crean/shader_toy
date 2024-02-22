pub mod polypeptide_plane;
pub mod polypeptide_planes;
pub mod protein_asset_loader;

use bevy::{
    app::{Plugin, Update},
    asset::{AssetApp, AssetEvent, Assets},
    ecs::{
        event::EventReader,
        system::{Commands, ResMut},
    },
    log::info,
    pbr::{PbrBundle, StandardMaterial},
    render::{color::Color, mesh::Mesh},
    utils::default,
};
use bevy_geometry::primitives::ribbon::Ribbon;

use protein_asset_loader::{ProteinAsset, ProteinAssetLoader};

pub struct ProteinPlugin;

impl Plugin for ProteinPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<ProteinAsset>()
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
                            polypeptide_planes, ..
                        }) => {
                            for plane_triptyc in polypeptide_planes.0.windows(8) {
                                //we need at least 4 controls points
                                let discrete_geodesic = vec![
                                    plane_triptyc[0].tangent_space,
                                    plane_triptyc[1].tangent_space,
                                    plane_triptyc[2].tangent_space,
                                    plane_triptyc[3].tangent_space,
                                    plane_triptyc[4].tangent_space,
                                    plane_triptyc[5].tangent_space,
                                    plane_triptyc[6].tangent_space,
                                    plane_triptyc[7].tangent_space,
                                ];

                                info!("{:?}", &discrete_geodesic);

                                let ribbon = Ribbon::new(discrete_geodesic, 1., 1., 50);

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

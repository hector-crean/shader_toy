pub mod pdb_asset_loader;
pub mod polypeptide_plane;
pub mod polypeptide_planes;
pub mod tangent_space;

use bevy::{
    app::{Plugin, Startup, Update},
    asset::{processor::LoadTransformAndSave, AssetApp, AssetEvent, AssetServer, Assets, Handle},
    ecs::{
        change_detection::DetectChanges,
        event::EventReader,
        system::{Command, Commands, In, Res, ResMut, Resource},
    },
    log::info,
    math::{
        cubic_splines::{CubicBSpline, CubicBezier, CubicGenerator, *},
        primitives::Plane3d,
        Vec3,
    },
    pbr::{MaterialMeshBundle, PbrBundle, StandardMaterial},
    render::{color::Color, mesh::Mesh},
    transform::components::Transform,
    utils::default,
};
use pdb_asset_loader::{ProteinAsset, ProteinAssetLoader};
use polypeptide_planes::PolypeptidePlanes;

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
                            let ribbon_mesh = polypeptide_planes.create_ribbon_mesh(2.);

                            let ribbon_mesh_handle = meshes.add(ribbon_mesh);

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

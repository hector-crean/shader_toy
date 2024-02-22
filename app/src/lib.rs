pub mod light_rig;
pub mod material;
pub mod state;

use std::f32::consts::PI;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    pbr::ExtendedMaterial,
    prelude::*,
    reflect::TypePath,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat, TextureUsages,
        },
    },
    window::PresentMode,
};
use bevy_asset_loader::prelude::*;
use bevy_cameras::{
    pan_orbit_camera::{OrbitCameraController, OrbitCameraControllerPlugin},
    CameraMode,
};
use bevy_mod_picking::{
    debug::DebugPickingPlugin, prelude::low_latency_window_plugin, DefaultPickingPlugins,
};
use bevy_protein::{
    polypeptide_plane::{PolypeptidePlane, PolypeptidePlaneError},
    polypeptide_planes::PolypeptidePlanes,
    protein_asset_loader::ProteinAsset,
    ProteinPlugin,
};
use light_rig::LightRigPlugin;
use material::{
    custom_material::CustomMaterial,
    extended_marerial::MyExtension,
    game_of_life::{GameOfLifeComputePlugin, GameOfLifeImage},
};
use pdbtbx::*;
use state::camera::CameraModeImpl;

#[derive(AssetCollection, Resource)]
struct ProteinAssetsMap {
    #[asset(path = "pdbs/AF-A0A7K5PA91-F1-model_v4.cif")]
    primary_protein: Handle<ProteinAsset>,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    AssetLoading,
    Main,
}

#[derive(Component)]
pub struct MainCamera;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..Default::default()
                })
                .set(low_latency_window_plugin()),
            DefaultPickingPlugins
                .build()
                .disable::<DebugPickingPlugin>(),
            OrbitCameraControllerPlugin::<CameraModeImpl>::default(),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
            MaterialPlugin::<CustomMaterial>::default(),
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, MyExtension>>::default(),
            ProteinPlugin,
            LightRigPlugin,
        ))
        .init_state::<AppState>()
        .add_loading_state(
            LoadingState::new(AppState::AssetLoading)
                .continue_to_state(AppState::Main)
                .load_collection::<ProteinAssetsMap>(),
        )
        .add_systems(OnEnter(AppState::Main), (Self::setup_camera,).chain());
    }
}

impl AppPlugin {
    fn setup_camera(mut commands: Commands) {
        commands.spawn((
            Camera3dBundle { ..default() },
            OrbitCameraController::new(100., Vec3::new(-2.6, -8.3, 9.0)),
            MainCamera,
        ));
    }
}

#[derive(Component)]
struct Rotate;

fn rotate_things(mut q: Query<&mut Transform, With<Rotate>>, time: Res<Time>) {
    for mut t in &mut q {
        t.rotate_y(time.delta_seconds());
    }
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn cleanup<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

fn print_resources(world: &World) {
    let components = world.components();

    let mut r: Vec<_> = world
        .storages()
        .resources
        .iter()
        .map(|(id, _)| components.get_info(id).unwrap())
        .map(|info| info.name())
        .collect();

    // sort list alphebetically
    r.sort();
    r.iter().for_each(|name| info!("{}", name));
}

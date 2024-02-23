pub mod light_rig;
pub mod material;
pub mod state;
use bevy_mod_picking::prelude::*;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::vec4,
    pbr::ExtendedMaterial,
    prelude::*,
};
use bevy_asset_loader::prelude::*;
use bevy_cameras::pan_orbit_camera::{OrbitCameraController, OrbitCameraControllerPlugin};
use bevy_mod_picking::{
    debug::DebugPickingPlugin, prelude::low_latency_window_plugin, DefaultPickingPlugins,
};
use bevy_protein::{protein_asset_loader::ProteinAsset, ProteinPlugin};
use light_rig::LightRigPlugin;
use material::{custom_material::CustomMaterial, extended_marerial::MyExtension};
use pdbtbx::*;
use state::camera::CameraModeImpl;

use bevy_instanced::plugin::InstancedMaterialPlugin;

#[derive(AssetCollection, Resource)]
struct ProteinAssetsMap {
    #[asset(path = "pdbs/AF-A0A2K5XT84-F1-model_v4.cif")]
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
            FrameTimeDiagnosticsPlugin,
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
        .add_systems(
            OnEnter(AppState::Main),
            (Self::setup_camera, make_pickable).chain(),
        );
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

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn cleanup<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

/// Makes everything in the scene with a mesh pickable
fn make_pickable(
    mut commands: Commands,
    meshes: Query<Entity, (With<Handle<Mesh>>, Without<Pickable>)>,
) {
    for entity in meshes.iter() {
        commands
            .entity(entity)
            .insert((PickableBundle::default(), HIGHLIGHT_TINT.clone()));
    }
}

const HIGHLIGHT_TINT: Highlight<StandardMaterial> = Highlight {
    hovered: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + Color::rgba(-0.2, -0.2, 0.4, 0.0),
        ..matl.to_owned()
    })),
    pressed: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + Color::rgba(-0.3, -0.3, 0.5, 0.0),
        ..matl.to_owned()
    })),
    selected: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + Color::rgba(-0.3, 0.2, -0.3, 0.0),
        ..matl.to_owned()
    })),
};

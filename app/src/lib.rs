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
use bevy_cameras::{
    pan_orbit_camera::{OrbitCameraController, OrbitCameraControllerPlugin},
    CameraMode,
};
use material::{
    custom_material::CustomMaterial,
    extended_marerial::MyExtension,
    game_of_life::{GameOfLifeComputePlugin, GameOfLifeImage},
};

use state::camera::CameraModeImpl;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Loading,
    Menu,
    Canvas3d,
}

#[derive(Component)]
pub struct MainCamera;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins.set(AssetPlugin {
                watch_for_changes_override: Some(true),
                ..Default::default()
            }),
            OrbitCameraControllerPlugin::<CameraModeImpl>::default(),
            GameOfLifeComputePlugin,
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
            MaterialPlugin::<CustomMaterial>::default(),
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, MyExtension>>::default(),
        ))
        .init_state::<AppState>()
        .add_systems(Startup, Self::setup)
        .add_systems(Update, rotate_things);
    }
}

impl AppPlugin {
    fn setup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut images: ResMut<Assets<Image>>,
        mut custom_materials: ResMut<Assets<CustomMaterial>>,
        mut extended_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, MyExtension>>>,
        asset_server: Res<AssetServer>,
    ) {
        commands.spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0., 2., 0.).looking_at(Vec3::ZERO, Vec3::Y),
                // projection: Projection::Orthographic(OrthographicProjection::default()),
                ..default()
            },
            OrbitCameraController::default(),
            MainCamera,
        ));

        const SIZE: (u32, u32) = (1280, 720);

        let mut image = Image::new_fill(
            Extent3d {
                width: SIZE.0,
                height: SIZE.1,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 0, 255],
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::RENDER_WORLD,
        );
        image.texture_descriptor.usage = TextureUsages::COPY_DST
            | TextureUsages::STORAGE_BINDING
            | TextureUsages::TEXTURE_BINDING;

        let image = images.add(image);

        // plane
        commands.spawn((MaterialMeshBundle {
            mesh: meshes.add(Plane3d::default()),
            transform: Transform::from_xyz(0.0, 0., 0.0),
            material: custom_materials.add(CustomMaterial {
                color: Color::RED,
                game_of_life_texture: Some(image.clone()),
                albedo_texture: Some(asset_server.load("images\\cell\\Albedo.png")),
                ao_texture: Some(asset_server.load("images\\cell\\AO.png")),
                normal_texture: Some(asset_server.load("images\\cell\\normal.png")),

                alpha_mode: AlphaMode::Blend,
            }),
            ..default()
        },));

        commands.insert_resource(GameOfLifeImage { texture: image });

        // light

        // commands.spawn((
        //     DirectionalLightBundle {
        //         transform: Transform::from_xyz(0., -1., 0.).looking_at(Vec3::ZERO, Vec3::Y),
        //         ..default()
        //     },
        //     Rotate,
        // ));

        // sphere
        // commands.spawn(MaterialMeshBundle {
        //     mesh: meshes.add(Plane3d::default()),
        //     transform: Transform::from_xyz(0.0, 0., 0.0),

        //     material: extended_materials.add(ExtendedMaterial {
        //         base: StandardMaterial {
        //             double_sided: true,

        //             base_color_texture: Some(asset_server.load("images\\cell\\Albedo.png")),
        //             normal_map_texture: Some(asset_server.load("images\\cell\\normal.png")),
        //             // can be used in forward or deferred mode.
        //             opaque_render_method: bevy::pbr::OpaqueRendererMethod::Auto,
        //             // in deferred mode, only the PbrInput can be modified (uvs, color and other material properties),
        //             // in forward mode, the output can also be modified after lighting is applied.
        //             // see the fragment shader `extended_material.wgsl` for more info.
        //             // Note: to run in deferred mode, you must also add a `DeferredPrepass` component to the camera and either
        //             // change the above to `OpaqueRendererMethod::Deferred` or add the `DefaultOpaqueRendererMethod` resource.
        //             ..Default::default()
        //         },
        //         extension: MyExtension { quantize_steps: 3 },
        //     }),
        //     ..default()
        // });
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

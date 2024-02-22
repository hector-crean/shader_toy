use bevy::prelude::*;
use std::f32::consts::PI;

pub struct LightRigPlugin;

impl Plugin for LightRigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (Self::ambient_light, Self::sunlight, Self::point_light),
        );
    }
}

impl LightRigPlugin {
    fn ambient_light(mut commands: Commands) {
        // ambient light
        commands.insert_resource(AmbientLight {
            color: Color::ORANGE_RED,
            brightness: 1.2,
        });
    }
    fn sunlight(mut commands: Commands) {
        // directional 'sun' light
        commands.spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 2.0, 0.0),
                rotation: Quat::from_rotation_x(-PI / 4.),
                ..default()
            },

            ..default()
        });
    }
    fn point_light(mut commands: Commands) {
        commands.spawn((PointLightBundle {
            point_light: PointLight {
                intensity: 5.0,
                shadows_enabled: true,
                ..Default::default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..Default::default()
        },));
    }
}

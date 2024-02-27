use crate::util::InternalShaders;
use bevy::{asset::embedded_asset, prelude::*};

pub const DEFAULT_EYE_DOME_PIPELINE_SHADER: &str =
    "embedded://bevy_pointcloud/render/shaders/eye_dome_pipeline/main.wgsl";
pub const DEFAULT_POINTCLOUD_PIPELINE_SHADER: &str =
    "embedded://bevy_pointcloud/render/shaders/pointcloud_pipeline/main.wgsl";

pub(crate) fn load_instancing_shaders(app: &mut App) {
    embedded_asset!(app, "eye_dome_pipeline/main.wgsl");
    embedded_asset!(app, "pointcloud_pipeline/main.wgsl");

    InternalShaders::load(
        app,
        &[
            DEFAULT_EYE_DOME_PIPELINE_SHADER,
            DEFAULT_POINTCLOUD_PIPELINE_SHADER,
        ],
    );
}

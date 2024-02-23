use crate::util::InternalShaders;
use bevy::{asset::embedded_asset, prelude::*};

pub const DEFAULT_SHADER: &str = "embedded://bevy_instanced/render/shaders/render/instancing.wgsl";

pub(crate) fn load_instancing_shaders(app: &mut App) {
    embedded_asset!(app, "render/instancing.wgsl");

    InternalShaders::load(
        app,
        &["embedded://bevy_instanced/render/shaders/render/instancing.wgsl"],
    );
}

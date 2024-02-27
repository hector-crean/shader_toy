use bevy::{
    prelude::*,
    render::{
        render_resource::{encase::internal::WriteInto, *},
        renderer::{RenderDevice, RenderQueue},
    },
};
use itertools::Itertools;
use std::{marker::PhantomData, ops::Deref};

#[derive(Default, Resource)]
pub(crate) struct InternalShaders(Vec<Handle<Shader>>);

impl InternalShaders {
    pub(crate) fn load(app: &mut App, shaders: &[&'static str]) {
        let mut shaders = shaders
            .iter()
            .map(|&shader| app.world.resource_mut::<AssetServer>().load(shader))
            .collect_vec();

        let mut internal_shaders = app.world.resource_mut::<InternalShaders>();
        internal_shaders.0.append(&mut shaders);
    }
}

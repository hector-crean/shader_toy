#define_import_path bevy_pointcloud::bindings

#import bevy_pointcloud::types::{Point, ClippingPlane, ClippingPlanes, Model, PointOffset, AnimationOffset1, AnimationOffset2, PointcloudAsset}


@group(1) @binding(0) var<uniform> clipping_planes : ClippingPlanes;
#ifdef MULTISAMPLED
@group(1) @binding(1) var input_texture: texture_multisampled_2d<f32>;
#else
@group(1) @binding(1) var input_texture: texture_2d<f32>;
#endif


@group(1) @binding(2) var<uniform> asset : PointcloudAsset;
#ifdef ANIMATED
@group(1) @binding(3) var<uniform> animationOffset1 : AnimationOffset1;
@group(1) @binding(4) var<uniform> animationOffset2 : AnimationOffset2;
#endif








#define_import_path bevy_pointcloud::types

struct Point {
    position: vec3<f32>;
    #ifdef COLORED
    color: vec4<f32>;
    #endif
};

struct ClippingPlane {
    origin : vec3<f32>;
    unit_normal : vec3<f32>;
    min_sdist : f32;
    max_sdist : f32;
};

struct ClippingPlanes {
    ranges : array<ClippingPlane>;
    num_ranges : u32;
};

struct Model {
    model_transform : mat4x4<f32>;
    point_size_world_space : f32;
};

struct PointOffset {
    offset: vec3<f32>;
};

struct AnimationOffset1 {
    _old_interpolation : f32;
    prev_offsets : array<PointOffset>;
};

struct AnimationOffset2 {
    interpolation : f32;
    next_offsets : array<PointOffset>;
};

struct PointcloudAsset {
    points : array<Point>;
};
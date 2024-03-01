#define_import_path bevy_volumetric::types


struct Triangle {
    a: vec3<f32>;
    b: vec3<f32>;
    c: vec3<f32>;
};

struct Cube {
    triangle_count: u32;
    triangles: array<Triangle, 5>;
};


struct Grid {
    cubes: array<Cube>
}







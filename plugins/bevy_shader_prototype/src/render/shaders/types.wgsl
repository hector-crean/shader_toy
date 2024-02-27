#define_import_path bevy_shader_prototype::types

struct Parameters {
    tile_count: u32,
    counter: i32,
    child_index: atomic<i32>,
    final_index: atomic<i32>,
}

struct Tile {
    st: vec3<f32> // [0..1]
    size: f32 //[0..1]
    side: u32 // [0...]
}

struct TileList {
    data: array<Chunk>
}

struct S3Coordinate {
    side: u32,
    st: vec3<f32>
}

struct Morph {
    ratio: f32,
}

struct Blend {
    lod: u32,
    ratio: f32,
}

struct OctreeEntry {
    atlas_index: u32,
    atlas_lod: u32,
}

struct Octree {
    data: array<OctreeEntry>,
}

struct LookupInfo {
    coordinate: S3Coordinate,
    view_distance: f32,
    lod: u32,
    blend_ratio: f32,
    // dr: mat3x3<f32>,
    ddx: vec3<f32>,
    ddy: vec3<f32>,
    ddz: vec3<f32>,
}

// A lookup of a node inside the node atlas based on the view of a octree.
struct NodeLookup {
    atlas_index: u32,
    atlas_lod: u32,
    atlas_coordinate: vec3<f32>,
    ddx: vec3<f32>,
    ddy: vec3<f32>,
    ddz: vec3<f32>,
    side: u32,
}



struct AttachmentConfig {
    size: f32,
    scale: f32,
    offset: f32,
    _padding: u32,
}


struct AttachmentList {
    data: array<AttachmentConfig, 8u>,
}




struct Triangle {
    a: vec3<f32>;
    b: vec3<f32>;
    c: vec3<f32>;
};

struct Cube {
    triangle_count: u32;
    triangles: array<Triangle, 5>;
};


struct Chunk {
    chunk_size: u32;
    position: vec3<f32>;
}
struct PolygonisedScalarField {
    data: array<Cube>
}





// :r = (x,y,z,d), where n=(x,y,z) is the normal vector to the plane d is the absolute distance from plane to the origin along the normal direction
type ClippingPlane = vec4<f32>; 


struct ClippingPlanes {
    planes: array<ClippingPlane>;
}
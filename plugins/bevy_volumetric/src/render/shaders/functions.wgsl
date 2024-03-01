
#import bevy_shader_prototype::types::{ClippingPlanes, ClippingPlane}


fn clip_plane(vertex_position: vec3<f32>, clipping_plane: vec4<f32>) {

    let distance = dot(vertex_position, clipping_plane.xyz) + clipping_plane.w;
    if distance < 0.0 {
        // This vertex is behind the clipping plane. You can manipulate it or discard it.
        // For example, you might set a flag or move the vertex far away.
    }
}


fn morton_encode(octree_coord: vec3<u32>) -> u32 {

    var morton_code: u32 = 0;

    for (var i: i32 = 0; i < 21; i = i + 1) {
        mortonCode = mortonCode | ((coord.x & (1 << i)) << (2 * i));
        mortonCode = mortonCode | ((coord.y & (1 << i)) << (2 * i + 1));
        mortonCode = mortonCode | ((coord.z & (1 << i)) << (2 * i + 2));
    }


    return 1
}

fn morton_decode(morton_encoding: u32) -> vec3<u32> {



    return 1
}
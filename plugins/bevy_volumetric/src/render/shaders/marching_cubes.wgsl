// Marching Cubes compute shader to polygonize a scalar field 

// The volume data is held within a 3D texture that is “animated” every frame by the first compute shader pass. 
// By default, a (sculpted) noise field is drawn. The next compute shader pass runs the marching cubes algorithm. 
// Specifically, it examines each cell (voxel) of the 3D volume texture and determines which (and how many) triangles 
// to draw. The resulting triangle data (positions and normals) is written into two write-only buffers, which are 
// subsequently used to draw a triangular mesh with a vertex + fragment shader. For now, the normals are calculated 
// by taking the gradient of the signed-distance function 

#import bevy_volumetric::constants::{EDGE_TABLE, TRI_TABLE, CORNER_INDEX_A_FROM_EDGE, CORNER_INDEX_B_FROM_EDGE}



struct VertexBuffer {
    data: array<vec3<f32>>,
};

struct NormalBuffer {
    data: array<vec3<f32>>,
};

struct IndexBuffer {
    data: array<u32>,
};

struct UvBuffer {
    data: array<vec2<f32>>,
};






@group(1) @binding(0) var vector_field: texture_3d<f32>;
@group(1) @binding(1) var vector_field_sampler: sampler;


@group(0) @binding(4)
var<storage, read_write> out_vertices: VertexBuffer;

@group(0) @binding(5)
var<storage, read_write> out_normals: NormalBuffer;

@group(0) @binding(6)
var<storage, read_write> out_indices: IndexBuffer;

@group(0) @binding(7)
var<storage, read_write> out_uvs: UvBuffer;




fn index_from_id(id: vec3<u32>, chunk: Chunk) -> u32 {
    return id.z * Chunk.chunk_size * Chunk.chunk_size + id.y * Chunk.chunk_size + id.x;
}





// @group(0) @binding(0) var texture: texture_storage_3d<r32float, read_write>;



@group(5) @binding(3) var<storage, read_write> polygonised_scalar_field: PolygonisedScalarField;

@compute @workgroup_size(8, 8, 8) 
fn marching_cubes(
    @builtin(global_invocation_id) id: vec3<u32>
) -> PolygonisedScalarField {


    var cube_corners: array<vec4<f32>, 8> = array<vec4<f32>, 8>(
        textureSample(vector_field, vector_field_sampler, vec3(id.x, id.y, id.z)),
        textureSample(vector_field, vector_field_sampler, vec3(id.x + 1u, id.y, id.z)),
        textureSample(vector_field, vector_field_sampler, vec3(id.x + 1u, id.y, id.z + 1u)),
        textureSample(vector_field, vector_field_sampler, vec3(id.x, id.y, id.z + 1u)),
        textureSample(vector_field, vector_field_sampler, vec3(id.x, id.y + 1u, id.z)),
        textureSample(vector_field, vector_field_sampler, vec3(id.x + 1u, id.y + 1u, id.z)),
        textureSample(vector_field, vector_field_sampler, vec3(id.x + 1u, id.y + 1u, id.z + 1u)),
        textureSample(vector_field, vector_field_sampler, vec3(id.x, id.y + 1u, id.z + 1u)),
    );


    var iso_level = 0.3;

    var cube_index = 0u;

    if cube_corners[0].w < iso_level { cube_index = cube_index | 1u; }
    if cube_corners[1].w < iso_level { cube_index = cube_index | 2u; }
    if cube_corners[2].w < iso_level { cube_index = cube_index | 4u; }
    if cube_corners[3].w < iso_level { cube_index = cube_index | 8u; }
    if cube_corners[4].w < iso_level { cube_index = cube_index | 16u; }
    if cube_corners[5].w < iso_level { cube_index = cube_index | 32u; }
    if cube_corners[6].w < iso_level { cube_index = cube_index | 64u; }
    if cube_corners[7].w < iso_level { cube_index = cube_index | 128u; }

    var triangle_index = 0u;
    var triangles: array<Triangle, 5>;

    for (var i = 0u; TRI_TABLE[cube_index][i] != -1; i = i + 3u) {
        var a0 = CORNER_INDEX_A_FROM_EDGE[u32(TRI_TABLE[cube_index][i])];
        var b0 = CORNER_INDEX_B_FROM_EDGE[u32(TRI_TABLE[cube_index][i])];

        var a1 = CORNER_INDEX_A_FROM_EDGE[u32(TRI_TABLE[cube_index][i + 1u])];
        var b1 = CORNER_INDEX_B_FROM_EDGE[u32(TRI_TABLE[cube_index][i + 1u])];

        var a2 = CORNER_INDEX_A_FROM_EDGE[u32(TRI_TABLE[cube_index][i + 2u])];
        var b2 = CORNER_INDEX_B_FROM_EDGE[u32(TRI_TABLE[cube_index][i + 2u])];

        triangles[triangle_index] = Triangle(
            interpolate_vertices(cube_corners[a0], cube_corners[b0], iso_level),
            interpolate_vertices(cube_corners[a1], cube_corners[b1], iso_level),
            interpolate_vertices(cube_corners[a2], cube_corners[b2], iso_level),
        );

        triangle_index = triangle_index + 1u;
    }

    polygonised_scalar_field.data[index_from_id(id, chunk)] = Cube(triangle_index, triangles);
}
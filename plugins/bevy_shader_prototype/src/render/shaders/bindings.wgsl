#define_import_path bevy_shader_prototype::bindings

#import bevy_shader_prototype::types::{ClippingPlanes}


// uniform values:
@group(1) @binding(1) var<uniform> LOD_COUNT: u32;
@group(1) @binding(2) var<uniform> NODE_COUNT: u32;
@group(1) @binding(3) var<uniform> VIEW_POSITION: vec4<f32>;
@group(1) @binding(4) var<uniform> FRUSTUM_PLANES: array<vec4<f32>, 6>;
@group(1) @binding(5) var<uniform> MAX_TILE_COUNT: u32;
@group(1) @binding(6) var<uniform> TILE_SCALE: f32;
@group(1) @binding(7) var<uniform> VERTICES_PER_TILE: u32;
@group(1) @binding(8) var<uniform> VERTICES_PER_ROW: u32;
@group(1) @binding(9) var<uniform> GRID_SIZE: f32;
@group(1) @binding(10) var<uniform> APPROXIMATE_HEIGHT: f32;
@group(1) @binding(11) var<uniform> MAX_HEIGHT: f32;
@group(1) @binding(12) var<uniform> LEAF_NODE_SIZE: f32;
@group(1) @binding(13) var<uniform> BLEND_DISTANCE: f32; // view_distance * leaf_node_size
@group(1) @binding(14) var<uniform> BLEND_RANGE: f32;
@group(1) @binding(15) var<uniform> MORPH_DISTANCE: f32; // view_distance * leaf_node_size
@group(1) @binding(16) var<uniform> MORPH_RANGE: f32;
// for each attachment:
@group(2) @binding(1) var<uniform> ATTACHMENT_SIZE: f32; // center_size + 2 * border_size
@group(2) @binding(2) var<uniform> ATTACHMENT_SCALE: f32; // center_size / ATTACHMENT_SIZE
@group(2) @binding(3) var<uniform> ATTACHMENT_OFFSET: f32; // border_size / ATTACHMENT_SIZE
@group(2) @binding(4) var attachment_atlas: texture_2d_array<f32>;
// prepass bindings
@group(3) @binding(1) var<storage, read_write> indirect_buffer: IndirectBuffer;
@group(3) @binding(2) var<storage, read_write> parameters: Parameters;
@group(3) @binding(3) var<storage, read_write> temporary_tiles: TileList;
@group(3) @binding(4) var<storage, read_write> final_tiles: TileList;
@group(3) @binding(5) var octree: texture_2d_array<u32>;
// render bindings
@group(4) @binding(1) var<storage> tiles: TileList;
@group(4) @binding(2) var octree: texture_2d_array<u32>;
@group(4) @binding(3) var atlas_sampler: sampler;












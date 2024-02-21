#import bevy_pbr::forward_io::{VertexOutput, Vertex}
#import bevy_pbr::view_transformations::position_world_to_clip
#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip, mesh_position_local_to_world, mesh_tangent_local_to_world, mesh_normal_local_to_world}
#import bevy_pbr::skinning::{skin_normals, skin_model}
#import bevy_pbr::morph::morph
#import bevy_pbr::mesh_view_bindings::globals

// we can import items from shader modules in the assets folder with a quoted path

@group(2) @binding(0) var<uniform> material_color: vec4<f32>;

@group(2) @binding(1) var albedo_texture: texture_2d<f32>;
@group(2) @binding(2) var albedo_sampler: sampler;

@group(2) @binding(3) var ao_texture: texture_2d<f32>;
@group(2) @binding(4) var ao_sampler: sampler;

@group(2) @binding(5) var normal_texture: texture_2d<f32>;
@group(2) @binding(6) var normal_sampler: sampler;

@group(2) @binding(7) var game_of_life_texture: texture_2d<f32>;
@group(2) @binding(8) var game_of_life_sampler: sampler;

const COLOR_MULTIPLIER: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 0.5);




#ifdef MORPH_TARGETS
fn morph_vertex(vertex_in: Vertex) -> Vertex {
    var vertex = vertex_in;
    let weight_count = bevy_pbr::morph::layer_count();
    for (var i: u32 = 0u; i < weight_count; i ++) {
        let weight = bevy_pbr::morph::weight_at(i);
        if weight == 0.0 {
            continue;
        }
        vertex.position += weight * morph(vertex.index, bevy_pbr,:: morph,:: position_offset, i);
#ifdef VERTEX_NORMALS
        vertex.normal += weight * morph(vertex.index, bevy_pbr,:: morph,:: normal_offset, i);
#endif
#ifdef VERTEX_TANGENTS
        vertex.tangent += vec4(weight * morph(vertex.index, bevy_pbr,:: morph,:: tangent_offset, i), 0.0);
#endif
    }
    return vertex;
}
#endif

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;

#ifdef MORPH_TARGETS
    var vertex = morph_vertex(vertex_no_morph);
#else
    var vertex = vertex_no_morph;
#endif

#ifdef SKINNED
    var model = skin_model(vertex.joint_indices, vertex.joint_weights);
#else
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416 .
    var model = get_model_matrix(vertex_no_morph.instance_index);
#endif

#ifdef VERTEX_NORMALS
#ifdef SKINNED
    out.world_normal = skin_normals(model, vertex.normal);
#else
    out.world_normal = mesh_normal_local_to_world(
        vertex.normal,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif
#endif

#ifdef VERTEX_POSITIONS
    out.world_position = mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);
#endif

#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif

#ifdef VERTEX_UVS_B
    out.uv_b = vertex.uv_b;
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_tangent_local_to_world(
        model,
        vertex.tangent,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif





    return out;
}






@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let light_intensity = 3.;
    let normal = textureSample(normal_texture, normal_sampler, mesh.uv);

    let albedo = textureSample(albedo_texture, albedo_sampler, mesh.uv);

    let light_dir = vec3(0.2, 0.4, 0.2);

    let base_color = light_intensity * dot(normal.rgb, light_dir) * albedo;

    let gof_mask = textureSample(game_of_life_texture, game_of_life_sampler, mesh.uv);
    let alive = vec4<f32>(f32(true));
    let aliveness = dot(gof_mask, alive);


    return base_color * clamp(aliveness, 0.5, 1.0);
}
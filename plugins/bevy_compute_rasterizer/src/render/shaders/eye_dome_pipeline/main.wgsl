

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vertex(
   in: VertexInput
) -> VertexOutput {

    var out: VertexOutput;

    out.clip_position = vec4(position * 2.0 - 1.0, 0.0, 1.0);

    return out;
}


@fragment
fn fragment(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    
    var ilocation = vec2<i32>(in.clip_position.xy);
    var log_depth: f32 = log2(textureLoad(input_texture, ilocation, 0).r);

    var response: f32 = 0.0;
    response += max(0.0, log_depth - log2(textureLoad(input_texture, vec2<i32>(ilocation.x + 1, ilocation.y), 0)).r);
    response += max(0.0, log_depth - log2(textureLoad(input_texture, vec2<i32>(ilocation.x - 1, ilocation.y), 0)).r);
    response += max(0.0, log_depth - log2(textureLoad(input_texture, vec2<i32>(ilocation.x, ilocation.y + 1), 0)).r);
    response += max(0.0, log_depth - log2(textureLoad(input_texture, vec2<i32>(ilocation.x, ilocation.y - 1), 0)).r);
    response /= 4.0;

    var shade = exp(-response * 300.0 * edl_strength);
    return vec4<f32>(0.0, 0.0, 0.0, shade);
}
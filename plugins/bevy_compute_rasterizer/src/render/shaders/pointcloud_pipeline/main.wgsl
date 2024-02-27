

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>
}


@vertex
fn main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;


   return out;
}




@fragment
fn main(in: VertexOutput) -> FragmentOutput {
    var output: FragmentOutput;


    return output;


}

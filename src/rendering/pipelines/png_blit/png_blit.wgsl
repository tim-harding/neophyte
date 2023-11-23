struct Constants {
    src_over_dst_width: f32,
}

@group(0) @binding(0)
var tex: texture_2d<f32>;
@group(0) @binding(1)
var tex_sampler: sampler;
var<push_constant> constants: Constants;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vec2<f32>(
        f32(in_vertex_index % 2u),
        f32(((in_vertex_index + 5u) % 6u) / 3u),
    );
    out.clip_position = vec4<f32>(
        out.uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0),        
        0.0,
        1.0
    );
    out.uv.x = out.uv.x * constants.src_over_dst_width;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSampleLevel(
        tex,
        tex_sampler,
        in.uv,
        0.0
    );
    return sample;
}

struct Constants {
    src_size: vec2<u32>,
    dst_size: vec2<u32>,
    transparent: f32,
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
    let two = vec2<u32>(2u, 2u);
    let offset = vec2<f32>((constants.dst_size - constants.src_size) / two);
    let src_size = vec2<f32>(constants.src_size);
    let dst_size = vec2<f32>(constants.dst_size);
    let position = (out.uv * src_size + offset) / dst_size;
    out.clip_position = vec4<f32>(
        position * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0),        
        0.0,
        1.0
    );
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
    let exp = vec4<f32>(2.2, 2.2, 2.2, 1.0);
    let a = 1.0 - (1.0 - sample.a) * constants.transparent;
    let premul = vec4<f32>(a, a, a, 1.0);
    return pow(sample, exp) * premul;
}

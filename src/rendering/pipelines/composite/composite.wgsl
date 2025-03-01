struct Info {
    src: vec2<f32>,
    src_sz: vec2<f32>,
    dst: vec2<f32>,
    dst_sz: vec2<f32>,
}

@group(0) @binding(0)
var tex: texture_2d<f32>;
@group(0) @binding(1)
var tex_sampler: sampler;
var<push_constant> info: Info;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let uv = vec2<f32>(
        f32(in_vertex_index % 2u),
        f32(((in_vertex_index + 5u) % 6u) / 3u),
    );
    let src_window = info.src_sz * vec2<f32>(1.0, 0.5);
    out.uv = (info.src + uv * src_window) / info.src_sz;
    out.clip_position = vec4<f32>(
        (vec2<f32>(uv) * src_window + info.dst) / info.dst_sz * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0),
        1.0,
        1.0
    );
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSampleLevel(
        tex,
        tex_sampler,
        in.uv,
        0.0
    );
}
